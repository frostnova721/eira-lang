# Weaves

Weaves are the base types available in Eira. Weaves are built from the combination of [strands](strands.md), each having their own properties! But you probably don't have to worry about that.

Currently, Eira provides 
- Num _(numbers)_
- Text _(string)_
- Truth _(boolean)_
- Sign _(structs)_
- Spell _(functions)_
- Deck _(lists)_


> A small insider info: These weaves used to have Weave at the end of their name, but was removed for convinience! It was like NumWeave, TextWeave...

You could say these are the foundation of world's best the type-system! /s

Weave is defined as a Enum and only the Deck, Sign and Spell contain values within it. Defined in [weave.rs](/src/compiler/types/weaves.rs)