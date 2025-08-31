use crate::frontend::{strands::{ADDITIVE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, MULTIPLICATIVE_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND}, tapestry::Tapestry};

pub struct Weave {
    tapestry: Tapestry,
    name: &'static str
}

pub const NumWeave: Weave = Weave {
    name: "NumWeave",
    tapestry: Tapestry::new(ADDITIVE_STRAND | SUBTRACTIVE_STRAND | ORDINAL_STRAND | MULTIPLICATIVE_STRAND | EQUATABLE_STRAND),
};

// /// Numbers
// pub const NUMWEAVE: u64 = ADDITIVE_STRAND | MULTIPLICATIVE_STRAND | ORDINAL_STRAND | EQUATABLE_STRAND;

// /// Boolean representation
// pub const TRUTHWEAVE: u64 = CONDITIONAL_STRAND | EQUATABLE_STRAND;

// /// String representation
// pub const TEXTWEAVE: u64 = INDEXIVE_STRAND | CONCATINABLE_STRAND | EQUATABLE_STRAND;
