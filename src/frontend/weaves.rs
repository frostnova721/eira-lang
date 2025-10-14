use std::collections::HashMap;

use crate::frontend::{strand::{ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND}, tapestry::Tapestry};

#[derive(Debug, Clone, PartialEq)]
pub struct Weave {
    pub tapestry: Tapestry,
    pub name: &'static str,
}

pub fn gen_weave_map() -> HashMap<String, Weave> {
    let mut weaves_map: HashMap<String, Weave> = HashMap::new();
    for i in get_weave_arr() {
        weaves_map.insert(i.name.to_string(), i);
    }
    weaves_map
}

fn get_weave_arr() -> [Weave; 5] {
    [NumWeave, TextWeave, TruthWeave, EmptyWeave, SpellWeave]
}

/// Represents numbers
pub const NumWeave: Weave = Weave {
    name: "NumWeave",
    tapestry: Tapestry::new(ADDITIVE_STRAND | SUBTRACTIVE_STRAND | ORDINAL_STRAND | MULTIPLICATIVE_STRAND | DIVISIVE_STRAND | EQUATABLE_STRAND),
};

/// Represents string
pub const TextWeave: Weave = Weave {
    name: "TextWeave",
    tapestry: Tapestry::new(CONCATINABLE_STRAND | INDEXIVE_STRAND | EQUATABLE_STRAND)
};

/// Represents boolean
pub const TruthWeave: Weave = Weave {
    name: "TruthWeave",
    tapestry: Tapestry::new(CONDITIONAL_STRAND | EQUATABLE_STRAND)
};

/// Void
pub const EmptyWeave: Weave = Weave {
    name: "EmptyWeave",
    tapestry: Tapestry::new(NO_STRAND),
};

pub const SpellWeave: Weave = Weave {
    name: "SpellWeave",
    tapestry: Tapestry::new(CALLABLE_STRAND),
};

// /// Numbers
// pub const NUMWEAVE: u64 = ADDITIVE_STRAND | MULTIPLICATIVE_STRAND | ORDINAL_STRAND | EQUATABLE_STRAND;

// /// Boolean representation
// pub const TRUTHWEAVE: u64 = CONDITIONAL_STRAND | EQUATABLE_STRAND;

// /// String representation
// pub const TEXTWEAVE: u64 = INDEXIVE_STRAND | CONCATINABLE_STRAND | EQUATABLE_STRAND;
