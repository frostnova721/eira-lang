# Eira

A Programming language with some MAGIC!

## Core Features

Eira is aimed to be:

- Typesafe
- Portable
- Decent syntax with some magical twist!

## Progress (Will be updated)

- [x] Mutable and Immutable Variables (access, modification)
- [x] Scopes (Realms in Eira)
- [x] Control flow (using fate (if), divert (else), while)
- [x] Function and Closures (spells!)
- [ ] Structs (signs! Work in progressss)

## Current State

> state: Going Smooth!

- count to 10mil in ~380ms (in my pc btw!)

    for comparison, js: ~39ms (v8 JIT hits different), dart: ~480ms (with 'dart run --snapshot=main.jit' command, java: ~337ms), for obvious reasons, im not comparing with the AOT compiled codes! (tests done by me btw)

- trying to design a mid type-like system!

## Eira's Own Weave System!!

Alright! Thought quite a while and came up with a weave system, as an alternative to the type system! (its 99% similar, but who cares!)

For intro, Weaves are made from strands, a set of them. And the strands are the basic behaviours of the operands.

for example....

presence of Additive strand on a Weave would mean that the weave can be undergone '+' or '-' operation with the same weave!

For starters, Eira will be providing 3 Weaves and 4 strands (as of now, will be increased once language evolves from the early stages), namely: NumWeave, TextWeave and TruthWeave. Representing the Numbers, String and Boolean!

and for strands, check the code (too lazy to type em all)!

## 🚧 Status

Eira is under active development by [@frostnova721](https://github.com/frostnova721). With an expected use for scripting too but mainly focused on general usage.

If all the stars align (and the bugs don't bite), Eira will evolve into a beautiful, usable language built by a amateur programmer!

## Demo Syntax? Okay

```Eira

// imports!
channel 'magic_forest/secret_knowledge';

// Like structs, acts as types
sign Magic {
    type: NumWeave,
    offensive: TruthWeave,
}

// Add methods to a sign
attune Magic {
    // Methods are called like class methods, e.g., fireMagic.nullify()
    spell nullify() {
        chant this + " got nullified!";
    }
}

// Classes = tomes
tome SuperSecretMagicTome {

    // Mutable values
    mark mana = 1;

    // Immutable values
    bind rank = "Noobie sorcerer";

    // Compile time constants
    seal dragons = 0;

    // Public functions [forge = public]
    forge spell fireBall():: Magic {
        chant "pooof .... BOOOM!";
        release cast Magic { type: 01, offensive: true }
    }

    // Private functions [secret = private]
    secret spell summon():: Magic {
        chant "something rose up!.... A DEMON????!!!";
        release cast Magic { type: 05, offensive: false }
    }
}
```

ehm ehm.... subject to changes btw! (this demo will be changed accordingly)

professional readme upon close to completion of the basic features!

## Building

Incase you want to test this language out, follow the steps

- Clone the repository
- Write your code in tests/test.eira file
- Run `cargo run`

    or provide the path to a custom ".eira" file as first argument if script is in a different directory (cargo run -- path_to_eira_file)

There you go. You are a mage now!!

## License

Project bound by the spell of **GPLv3**. In mortal words: you may **fork, clone, edit, and maintain** - just don’t close-source your modifications.
