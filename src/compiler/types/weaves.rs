use std::{collections::HashMap, iter};

use crate::compiler::{
    strand::{
        ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, DIVISIVE_STRAND,
        EQUATABLE_STRAND, INDEXIVE_STRAND, ITERABLE_STRAND, MULTIPLICATIVE_STRAND, NO_STRAND,
        ORDINAL_STRAND, SUBTRACTIVE_STRAND,
    },
    tapestry::Tapestry,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Weave {
    Num,
    Text,
    Truth,
    Spell {
        reagents: Vec<Weave>,
        release: Box<Weave>,
    },
    Sign(String /* name */),
    Deck(Box<Weave>),
    Empty,
}

impl Weave {
    pub fn can_sub_weave(&self) -> bool {
        matches!(self, Weave::Spell { .. } | Weave::Deck(_))
    }

    pub fn get_tapestry(&self) -> Tapestry {
        match self {
            Weave::Num => Tapestry::new(
                ADDITIVE_STRAND
                    | SUBTRACTIVE_STRAND
                    | ORDINAL_STRAND
                    | MULTIPLICATIVE_STRAND
                    | DIVISIVE_STRAND
                    | EQUATABLE_STRAND,
            ),
            Weave::Text => Tapestry::new(CONCATINABLE_STRAND | INDEXIVE_STRAND | EQUATABLE_STRAND),
            Weave::Truth => Tapestry::new(CONDITIONAL_STRAND | EQUATABLE_STRAND),
            Weave::Empty => Tapestry::new(NO_STRAND),
            Weave::Spell { .. } => Tapestry::new(CALLABLE_STRAND),
            Weave::Sign(_) => Tapestry::new(NO_STRAND),
            Weave::Deck(_) => Tapestry::new(INDEXIVE_STRAND | ITERABLE_STRAND),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Weave::Num => "NumWeave".to_string(),
            Weave::Text => "TextWeave".to_string(),
            Weave::Truth => "TruthWeave".to_string(),
            Weave::Empty => "EmptyWeave".to_string(),
            Weave::Spell { .. } => "SpellWeave".to_string(),
            Weave::Sign(name) => format!("SignWeave<{}>", name),
            Weave::Deck(inner) => format!("DeckWeave<{}>", inner.get_name()),
        }
    }
}


// #[derive(Debug, Clone, Eq, PartialEq, Hash)]
// pub struct Weave {
//     /// The base tapestry (without sub-weaves)
//     pub base_tapestry: Tapestry,

//     /// The tapestry of the current weave
//     pub tapestry: Tapestry,

//     /// Whether this weave can have sub-weaves
//     pub can_sub_weave: bool,

//     /// The name of the weave
//     pub name: String,
// }

// pub fn gen_weave_map() -> HashMap<String, Weave> {
//     let mut weaves_map: HashMap<String, Weave> = HashMap::new();
//     for i in get_weave_arr() {
//         weaves_map.insert(i.get_name(), i);
//     }
//     weaves_map
// }

// fn get_weave_arr() -> [Weave; 7] {
//     Weave::iter
// }

#[derive(Debug, Clone)]
pub struct WeaverError(pub String);

type WeaverResult<T> = Result<T, WeaverError>;

pub struct Weaver();
impl Weaver {
    pub fn weave(base: Weave, inner: Weave) -> WeaverResult<Weave> {
        match base {
            Weave::Spell { reagents, release } => {
                let mut new_reagents = reagents.clone();
                new_reagents.push(inner);
                Ok(Weave::Spell {
                    reagents: new_reagents,
                    release,
                })
            }
            Weave::Deck(_) => Ok(Weave::Deck(Box::new(inner))),
            _ => Err(WeaverError(format!(
                "The weave '{}' cannot contain any sub weaves!",
                base.get_name()
            ))),
        }
    }
}
