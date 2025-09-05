/// Addition / Subtraction behaviour
pub const ADDITIVE_STRAND: u64 = 1 << 0;
pub const SUBTRACTIVE_STRAND: u64 = 1 << 1;

/// Multiplication / Division behaviour
pub const MULTIPLICATIVE_STRAND: u64 = 1 << 2;
pub const DIVISIVE_STRAND: u64 = 1 << 3;

/// Orderable behaviour (>,<,>=,<=)
pub const ORDINAL_STRAND: u64 = 1 << 4;

/// Passable to conditional statements, (if, while)
pub const CONDITIONAL_STRAND: u64 = 1 << 5;

/// Able to concat with same type
pub const CONCATINABLE_STRAND: u64 = 1 << 6;

/// Able to be indexed, pick a value from a index
pub const INDEXIVE_STRAND: u64 = 1 << 7;

/// Iterable...
pub const ITERABLE_STRAND: u64 = 1 << 8;

/// Able to be equated, ==, !=
pub const EQUATABLE_STRAND: u64 = 1 << 9;

/// Callable items (spells, methods)
pub const CALLABLE_STRAND: u64 = 1 << 10;

// pub const NEGATE_STRAND

// Emptiness
pub const NO_STRAND:u64 = 0;

pub struct Strand {
    pub name: String,
    pub value: u64
}