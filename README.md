# Eira

A Programming language with some MAGIC!

## Core Features

- Typesafe
- Portable
- Decent syntax with some magical twist!

## Progress (Will be updated)

- Very basic skeleton [100%]
- Grammar/Syntax planning [~80%]
- VM Design [~70%] (i was optimising the current one)

## Current State

> state: confused

- count to 10mil in ~380ms (in my pc btw!)
- rewriting the compiler to be split as parser, type check!
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

    // Public functions
    forge spell fireBall(): Magic {
        chant "pooof .... BOOOM!";
        release cast Magic { type: 01, offensive: true }
    }

    // Private functions
    secret spell summon(): Magic {
        chant "something rose up!.... A DEMON????!!!";
        release cast Magic { type: 05, offensive: false }
    }
}
```

ehm ehm.... subject to changes btw!

professional readme upon close to completion of the basic features!

## License

This project is bound to the spell **GPLv3 License**! <br>
Basically: you're free to **fork, clone, edit, and maintain** — just **don’t close-source** your modified version.
