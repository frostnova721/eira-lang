/// Addition / Subtraction behaviour
pub const ADDITIVE_STRAND: u64 = 1 << 0;

/// Multiplication / Division behaviour
pub const MULTIPLICATIVE_STRAND: u64 = 1 << 1;

/// Orderable behaviour (>,<,>=,<=)
pub const ORDINAL_STRAND: u64 = 1 << 2;

/// Passable to conditional statements, (if, while)
pub const CONDITIONAL_STRAND: u64 = 1 << 3;

/// Able to concat with same type
pub const CONCATINABLE_STRAND: u64 = 1 << 4;

/// Able to be indexed, pick a value from a index
pub const INDEXIVE_STRAND: u64 = 1 << 5;

/// Iterable...
pub const ITERABLE_STRAND: u64 = 1 << 6;

/// Able to be equated, ==, !=
pub const EQUATABLE_STRAND: u64 = 1 << 7;

/// Callable items (spells, methods)
pub const CALLABLE_STRAND: u64 = 1 << 8;