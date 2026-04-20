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
        release: Box<Weave>,
    },
    Sign(String /* name */),
    Deck(Box<Weave>, Option<usize>),
    Empty,
}

impl Weave {
    pub fn can_sub_weave(&self) -> bool {
        matches!(self, Weave::Spell { .. } | Weave::Deck(_, _))
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
            Weave::Deck(_, _) => Tapestry::new(INDEXIVE_STRAND | ITERABLE_STRAND),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Weave::Num => "Num".to_string(),
            Weave::Text => "Text".to_string(),
            Weave::Truth => "Truth".to_string(),
            Weave::Empty => "Empty".to_string(),
            Weave::Spell { .. } => "Spell".to_string(),
            Weave::Sign(name) => format!("Sign<{}>", name),
            Weave::Deck(inner, length) => {
                let str = if length.is_some() {
                    &format!(", {}", length.unwrap())
                } else {
                    ""
                };
                format!("Deck<{}{}>", inner.get_name(), str)
            }
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
    pub fn weave_spell(base: Weave, inner: Weave) -> WeaverResult<Weave> {
        match base {
            Weave::Spell { .. } => {
                // let mut new_reagents = reagents.clone();
                // new_reagents.push(inner);
                Ok(Weave::Spell {
                    // reagents: new_reagents,
                    release: Box::new(inner),
                })
            }
            _ => Err(WeaverError(format!(
                "The weave '{}' cannot contain any sub weaves!",
                base.get_name()
            ))),
        }
    }

    pub fn weave_deck(base: Weave, inner: Weave, capacity: Option<usize>) -> WeaverResult<Weave> {
        match base {
            Weave::Deck(_, _) => Ok(Weave::Deck(Box::new(inner), capacity)),
            _ => Err(WeaverError(format!(
                "The weave '{}' cannot contain any sub weaves!",
                base.get_name()
            ))),
        }
    }
}
