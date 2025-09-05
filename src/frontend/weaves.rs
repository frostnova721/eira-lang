use crate::frontend::{strands::{ADDITIVE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, MULTIPLICATIVE_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND}, tapestry::Tapestry};

pub struct Weave {
    pub tapestry: Tapestry,
    pub name: &'static str
}

pub const NumWeave: Weave = Weave {
    name: "Rune",
    tapestry: Tapestry::new(ADDITIVE_STRAND | SUBTRACTIVE_STRAND | ORDINAL_STRAND | MULTIPLICATIVE_STRAND | DIVISIVE_STRAND | EQUATABLE_STRAND),
};

pub const TextWeave: Weave = Weave {
    name: "Script",
    tapestry: Tapestry::new(CONCATINABLE_STRAND | INDEXIVE_STRAND)
};

pub const TruthWeave: Weave = Weave {
    name: "Omen",
    tapestry: Tapestry::new(CONDITIONAL_STRAND | EQUATABLE_STRAND)
};

// /// Numbers
// pub const NUMWEAVE: u64 = ADDITIVE_STRAND | MULTIPLICATIVE_STRAND | ORDINAL_STRAND | EQUATABLE_STRAND;

// /// Boolean representation
// pub const TRUTHWEAVE: u64 = CONDITIONAL_STRAND | EQUATABLE_STRAND;

// /// String representation
// pub const TEXTWEAVE: u64 = INDEXIVE_STRAND | CONCATINABLE_STRAND | EQUATABLE_STRAND;
