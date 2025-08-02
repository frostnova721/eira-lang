# Eira

A Programming language with some MAGIC!

## Core Features

- Typesafe
- Portable
- Decent syntax with some magical twist!

## Progress (Will be updated)

- Very basic skeleton [100%]
- Grammar/Syntax planning [~20%]
- VM Design [~70%]

🚧 Status
Eira is under active development by [@frostnova721](https://github.com/frostnova721). With an expected use for scripting too but mainly focused on general usage.

If all the stars align (and the bugs don't bite), Eira will evolve into a beautiful, usable language built by a amateur programmer!

## Demo Syntax? Okay

```Eira

// imports!
channel 'magic_forest/secret_knowledge';

// Like structs, acts as types
sign Magic {
    type: Int,
    offensive: Bool,
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

    // Constants
    seal dragons = 0;

    // Public functions
    forge spell fireBall(): Magic {
        chant "pooof .... BOOOM!";
        // returns arent designed yet :(
    }

    // Private functions
    secret spell summon(): Magic {
        chant "something rose up!.... A DEMON????!!!";
    }
}
```

ehm ehm.... subject to changes btw!

professional readme upon close to completion of the basic features!

## License

This project is bound to the spell **GPLv3 License**! <br>
Basically: you're free to **fork, clone, edit, and maintain** — just **don’t close-source** your modified version.
