use std::{collections::HashMap};

use crate::frontend::{
    strand::{
        ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, DIVISIVE_STRAND,
        EQUATABLE_STRAND, INDEXIVE_STRAND, MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND,
        SUBTRACTIVE_STRAND,
    },
    tapestry::Tapestry,
};

pub enum Weaves {
    NumWeave,
    TextWeave,
    TruthWeave,
    SpellWeave,
    EmptyWeave,
}

impl Weaves {
    pub fn get_weave(&self) -> Weave {
        match self {
            Weaves::NumWeave => num_weave(),
            Weaves::TextWeave => text_weave(),
            Weaves::TruthWeave => truth_weave(),
            Weaves::SpellWeave => spell_weave(),
            Weaves::EmptyWeave => empty_weave(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Weave {
    /// The base tapestry (without sub-weaves)
    pub base_tapestry: Tapestry,

    /// The tapestry of the current weave
    pub tapestry: Tapestry,

    /// Whether this weave can have sub-weaves
    pub can_sub_weave: bool,

    /// The name of the weave
    pub name: String,
}

pub fn gen_weave_map() -> HashMap<String, Weave> {
    let mut weaves_map: HashMap<String, Weave> = HashMap::new();
    for i in get_weave_arr() {
        weaves_map.insert(i.name.to_string(), i);
    }
    weaves_map
}

fn get_weave_arr() -> [Weave; 5] {
    [num_weave(), text_weave(), truth_weave(), empty_weave(), spell_weave()]
}

/// Represents numbers
fn num_weave() -> Weave {
    Weave {
        name: "NumWeave".to_string(),
        tapestry: Tapestry::new(
            ADDITIVE_STRAND
                | SUBTRACTIVE_STRAND
                | ORDINAL_STRAND
                | MULTIPLICATIVE_STRAND
                | DIVISIVE_STRAND
                | EQUATABLE_STRAND,
        ),
        can_sub_weave: false,
        base_tapestry: Tapestry::new(
            ADDITIVE_STRAND
                | SUBTRACTIVE_STRAND
                | ORDINAL_STRAND
                | MULTIPLICATIVE_STRAND
                | DIVISIVE_STRAND
                | EQUATABLE_STRAND,
        ),
    }
}

/// Represents string
fn text_weave() -> Weave {
    Weave {
        name: "TextWeave".to_string(),
        tapestry: Tapestry::new(CONCATINABLE_STRAND | INDEXIVE_STRAND | EQUATABLE_STRAND),
        base_tapestry: Tapestry::new(CONCATINABLE_STRAND | INDEXIVE_STRAND | EQUATABLE_STRAND),
        can_sub_weave: false,
    }
}

/// Represents boolean
fn truth_weave() -> Weave {
    Weave {
        name: "TruthWeave".to_string(),
        tapestry: Tapestry::new(CONDITIONAL_STRAND | EQUATABLE_STRAND),
        base_tapestry: Tapestry::new(CONDITIONAL_STRAND | EQUATABLE_STRAND),
        can_sub_weave: false,
    }
}

/// Void
fn empty_weave() -> Weave {
    Weave {
        name: "EmptyWeave".to_string(),
        tapestry: Tapestry::new(NO_STRAND),
        base_tapestry: Tapestry::new(NO_STRAND),
        can_sub_weave: false,
    }
}

fn spell_weave() -> Weave {
    Weave {
        name: "SpellWeave".to_string(),
        tapestry: Tapestry::new(CALLABLE_STRAND),
        base_tapestry: Tapestry::new(CALLABLE_STRAND),
        can_sub_weave: true,
    }
}

#[derive(Debug, Clone)]
pub struct WeaverError(pub String);

type WeaverResult<T> = Result<T, WeaverError>;

pub struct Weaver();
impl Weaver {
    pub fn weave(base: Weave, inner: Weave) -> WeaverResult<Weave> {
        if base.can_sub_weave {
            let mut new_tape = base.clone().tapestry;
            new_tape.weave(inner.tapestry.0);
            return Ok(Weave {
                name: format!("{}<{}>", base.name, inner.name),
                can_sub_weave: true,
                tapestry: new_tape,
                base_tapestry: base.base_tapestry.clone(),
            });
        }
        Err(WeaverError(format!("The weave '{}' cannot contain any sub weaves!", base.name)))
    }
}

// /// Numbers
// pub const NUMWEAVE: u64 = ADDITIVE_STRAND | MULTIPLICATIVE_STRAND | ORDINAL_STRAND | EQUATABLE_STRAND;

// /// Boolean representation
// pub const TRUTHWEAVE: u64 = CONDITIONAL_STRAND | EQUATABLE_STRAND;

// /// String representation
// pub const TEXTWEAVE: u64 = INDEXIVE_STRAND | CONCATINABLE_STRAND | EQUATABLE_STRAND;
