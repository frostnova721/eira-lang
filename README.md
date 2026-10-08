# Eira

A Programming language with some MAGIC!

Functions? **Spells!** Structs? **Signs!** Types? We call those **weaves** around here.
Eira is a language written in Rust, with its own compiler and bytecode VM doing the
magic behind the curtains.

Still early days, btw! The spells are taking shape, but syntax can change and a few
curses remain. If all the stars align (and the bugs don't bite), Eira will evolve
into a beautiful, usable language built by an amateur programmer!

## Summoning Eira (Building)

First, gather your reagents: Rust and Cargo with support for Rust edition 2024.
Then, from the repository root, cast:

```sh
cargo build --locked
cargo run --locked -- eira_scripts/factorial.eira
```

The factorial spell chants `120`. Got a scroll of your own? Save it as `hello.eira`:

```sh
cargo run --locked -- ./hello.eira
```

For a little more speed, summon a release build:

```sh
cargo run --locked --release -- eira_scripts/factorial.eira
```

Provide a source path when running from the repository root. With no source argument,
Eira looks for an `essence.toml` project configuration and uses its entry point;
the repository root does not contain one.

There you go. You are a mage now!!

## Demo Syntax? Okay

```eira
// A little mana and a greeting for our mage
mark mana: Num = 3;
bind greeting = "Hello, mage!";
chant greeting;

while mana > 0 {
    chant mana;
    mana = mana - 1;
}

spell double(n: Num):: Num {
    release n * 2;
}

chant cast double with 21;

sign Sword {
    material: Text,
}

attune Sword {
    spell describe() {
        chant ego.material;
    }
}

mark sword = ~Sword with {
    material: "Phoenix Steel",
};
cast sword.describe;

bind numbers: Deck<Num> = [1, 2, 3];
numbers[0] = 10;
chant numbers[0];
```

`bind` prevents reassignment of the binding; it does not make a deck's contents
immutable. `ego` refers to the receiver inside an attunement method.

More scrolls await in [eira_scripts](eira_scripts): recursion, signs, attunements,
and decks. The deck example deliberately overflows its capacity at the end. That
curse is part of the demonstration!

## Taking the runes for a spin (Iterator loops)

No index bookkeeping for this ritual! A **cycle** binds each value to the name
before `~~`, then casts the body once for each value:

```eira
// Let each rune have its turn to chant!
rune ~~ ["ember", "frost", "storm"] {
    chant rune;
}

// Both ends of the range join the ritual.
step ~~ 1..3 {
    chant step;
}
```

This chants `ember`, `frost`, `storm`, then `1`, `2`, `3`, each on its own line.
Decks are visited in order. Ranges include both endpoints and advance by one;
an empty deck or a range whose start exceeds its end runs the body zero times.
The cycle variable belongs to the loop's scope. Cycles currently accept decks
and ranges.

Cast the complete [iterator scroll](eira_scripts/iterators.eira) with:

```sh
cargo run --locked -- eira_scripts/iterators.eira
```

## Spells we have mastered (Progress)

- Mutable (`mark`) and immutable (`bind`) bindings, with scopes and weave annotations.
- Numbers, text, truth values, arithmetic, comparisons, and text concatenation.
- Branches with `fate` / `divert`; loops with `while`, `sever` (break), and `flow` (continue).
- Iterator cycles with `item ~~ deck` or `item ~~ start..end`.
- Spells, arguments, return values through `release`, recursion, and captured variables.
- Signs, field access and mutation, and methods declared with `attune`.
- Dynamic decks (`Deck<Num>`) and decks with a fixed capacity (`Deck<Num, 5>`).
- Partial imports through `tether`, including file paths and namespaces.
- Compilation diagnostics collected through `Augury`.

Still in the spellbook's unwritten pages: classes (`tome`), language-level error
handling, and external package dependencies. Native spells now have a registry and methods on numbers, text, and decks.
`Maybe<W>` support is still experimental.

## Eira's Own Weave System

Alright! Eira has a weave system, our magical twist on a type system. Weaves are
built from strands, which describe the behaviours a value supports. Familiar idea,
a little extra magic. Who says types can't have some personality?

And we've got more than just numbers, strings, and booleans in this spellbook:

| Weave | Meaning |
| --- | --- |
| `Num` | Numbers |
| `Text` | Text values |
| `Truth` | Boolean values: `true` and `false` |
| `Sign` | User-defined structures; use the declared sign's name, such as `Sword`, in annotations |
| `Spell<W>` | A spell whose release (return) weave is `W`; for example, `Spell<Num>` |
| `Deck<W>` | A dynamic deck containing values of weave `W` |
| `Deck<W, N>` | A deck with element weave `W` and a fixed capacity of `N` |
| `Maybe<W>` | A value that may contain `W` or be empty |
| `Range` | An inclusive numeric range, written `start..end`, that cycles can traverse |
| `Empty` | The absence of a value; also the default release weave for spells |
| `Module` | The compiler's representation of an imported namespace, introduced through `tether` |

`W` stands for an inner weave, and `N` is a numeric capacity. `Deck<W>` and
`Deck<W, N>` are two forms of the same deck weave. Module weaves are created by
import analysis rather than written as a `Module` annotation.

Strands describe capabilities used during semantic analysis. Numbers carry
arithmetic, ordering, and equality strands; text carries concatenation, indexing,
and equality strands; truth values carry conditional and equality strands. Spells
carry the callable strand, decks carry indexing and iterable strands, and
`Maybe<W>` carries presence and equality strands. Ranges carry iterable and
ordering strands. Signs, modules, and `Empty`
currently have no strands assigned.

The weave analyzer makes sure the strands line up and the names resolve before
we start generating instructions. For the full enchantment, peek into
[weaves.rs](src/compiler/types/weaves.rs).

How does a scroll become something the VM can cast? Through these stages:

```text
Source → Scanner → Parser → Weave analyzer → Code generation → Assembler → Bytecode VM
```

The [grimoire](grimoire/src/SUMMARY.md) has more notes on the design. A few pages
may be dusty; the tests and runnable scrolls are the best references for what
actually works today.

## Tethering scrolls (Imports)

A file can export a spell with `forge`. For example, `library.eira`:

```eira
forge spell answer():: Num {
    release 42;
}
```

A sibling `main.eira` can import it into a namespace:

```eira
tether "library.eira" bind lib;
chant cast lib.answer;
```

Use `./main.eira` or an absolute source path when running this example. Passing just
`main.eira` currently causes sibling imports to resolve from the filesystem root.
Imported files permit declarations at the top level, but not executable statements
such as `chant`. Named project imports and external dependencies remain incomplete.

## Peeking behind the magic (Compiler inspection)

Pass options after Cargo's `--`, alongside the source path:

```sh
cargo run --locked -- eira_scripts/factorial.eira --no-run --pinst
```

| Option | Purpose |
| --- | --- |
| `--no-run` | Compile without executing |
| `--ptkn` | Print tokens |
| `--past=N` | Print the parsed AST at verbosity `N` |
| `--pwast=N` | Print the analyzed AST at verbosity `N` |
| `--pinst` | Print instructions |
| `--pbc` | Print bytecode |

## Making sure the spells hold (Tests)

```sh
cargo test --locked
```

Current tally: **68 passing tests and 3 ignored regression tests**. Three known curses
have their own reproducers waiting for a fix; they aren't counted as victories!
The suite covers
scanning, parsing, weave analysis, compilation diagnostics, and program execution.
Tests are split by responsibility, with shared temporary-file setup.

See [tests/README.md](tests/README.md) for the layout, focused test commands, and
instructions for running the known regression cases.

## Known curses (Limitations)

- Nested iterator cycles can overwrite the outer iteration value and crash the VM;
  a regression test records the expected behavior.

- Returned closures can inherit the enclosing spell's argument metadata, causing
  valid calls to be rejected. A regression test captures this case.
- Bare CLI filenames break sibling import resolution, as described above.
- Compilation and runtime errors currently return exit status `0`; automation must
  inspect diagnostics rather than relying on exit status alone.
- The build currently emits warnings, and some implementation paths remain unfinished.

One old curse seems broken: the conditional-return crash in [issues.md](issues.md)
no longer reproduces in the current regression test. The old issue note still needs
catching up with the spellbook.

## License

Project bound by the spell of **GPLv3**. In mortal words: you may **fork, clone, edit, and maintain** - just don’t close-source your modifications.
