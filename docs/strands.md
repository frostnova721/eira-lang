# Strands

Strand are the basic functional units which corresponds or represents a behaviour.

As a user, you might not have to understand or use the strands, since its the internal component

Each strand is defined as an unsigned 64 bit integer. Each strands signifies a kind of behaviour.

ADDITIVE: Ability to undergo addition
SUBTRACTIVE: Ability to undergo subtraction
MULTIPLICATIVE: Ability to undergo multiplication
DIVISIVE: Ability to undergo division
CONCATINABLE: Ability to undergo concatination (usually for strings)

strands are defined and documented at [strand.rs](/src/compiler/types/strand.rs)