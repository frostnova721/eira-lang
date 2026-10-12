# Spells (Functions)

A **Spell** is a reusable block of magic which can perform a defined action. They can be casted whenever you want to execute the logic!

## Declaring a spell

For declaring a spell, the keyword `spell` is used. followed by its name, its **Reagents** (parameters) and optionally the **Release** (return) weave which would default to Empty.

```eira
// a spell with reagents and release value
spell candy_rain(duration: Num):: Truth {
    chant "candy rained for @duration hours!";
    release true;
}

// a spell without reagents and release value
spell invisible_rain() {
    chant "someting rained, for a split second!";
}
```

### Breakdown

Let's take the example of spell "candy" from the above snippet.

- Spell name: "candy"
- Reagents: duration with weave "Num"
- Release Weave: The weave defined after the double colon '::'.
- Release value: The 'true' is a value of weave 'Truth'

## Casting of spells (invokation)

A spell alone does not do anything, we need to cast it to affect the course of the fate.

In regular programming terms, its called function calling or function invokation

The following snippets shows how the spells can be casted.

```eira
// casts the spell 'candy_rain' with argument '2'
cast candy_rain with 2;

// casts the spell with argument '4' and saves the results to a variable
bind did_it_rain = cast candy_rain with 4; 

// casting without reagents
cast invisible_rain;
```

alternatively, you can use the **quick casts**. The syntax for it is same as you call functions 
in other languages...

```eira
bind sweeet = candy_rain(3);
```

NB: Quick casts was made so that you dont get into bloat code like _"(cast (cast gamer).play).score"_
situations.. As a great wizard (or witch..), thou shall use the normal casts for non chained castings!

## Native spells

Some spells come with Eira itself, no declaration or tethering needed! These are the **native spells**.
You cast them just like any other spell.

Here are all the standalone native spells currently in the spellbook:

| Spell | Reagents | Release weave | What it does |
| --- | --- | --- | --- |
| `listen` | None | `Text` | Reads a line from the terminal |
| `ask` | Prompt: `Text` | `Text` | Prints a prompt, then reads a line |
| `parseNum` | Value: `Text` | `Num` | Converts text to a number |
| `floor` | Value: `Num` | `Num` | Rounds down to the nearest whole number |
| `ceil` | Value: `Num` | `Num` | Rounds up to the nearest whole number |

### listen: Hear the mage out

`listen` waits for a line of input and releases it as Text. No reagents needed!

```eira
chant "What is your name, mage?";
bind mage = cast listen;
chant "Welcome, @mage!";
```

### ask: A question with its answer

`ask` takes a Text prompt and prints it without adding a new line. Then it reads
the answer, just like `listen`.

```eira
bind answer = cast ask with "How much mana? ";
bind mana = cast parseNum with answer;
chant mana + 10;
```

Both input spells remove surrounding whitespace, including the final newline.
An empty line releases empty Text. Reaching the end of input also releases empty
Text; a read failure causes a runtime error.

### parseNum: Text to Num

Got a number trapped inside a Text? `parseNum` gets it out!
It takes one Text reagent and releases a Num.

```eira
bind mana = cast parseNum with "42";
chant mana + 8; // 50

// decimals are numbers too!
bind potion_cost = cast parseNum with "12.5";
chant potion_cost; // 12.5

// or use a quick cast
chant parseNum("-7"); // -7
```

Spaces around the number are fine. Signs and scientific notation work too:

```eira
chant parseNum("  +25  "); // 25
chant parseNum("1.5e2"); // 150
```

The whole Text must be a number. `"12 potions"`, `"hello"` and even `""` will cause
a runtime error and stop the scroll. No guessing what the mage meant!
Passing a Num instead of Text, or giving the wrong number of reagents, is a compile error.

NB: Num uses floating-point numbers, so `parseNum` also accepts `"NaN"`, `"inf"`
and `"infinity"`. Very large numbers can overflow to infinity.

### floor and ceil: Whole numbers, please!

Both spells take one Num reagent and release a Num. `floor` rounds towards
negative infinity, while `ceil` rounds towards positive infinity.

```eira
chant cast floor with 4.8; // 4
chant cast ceil with 4.2; // 5

// watch the direction when the number is negative!
chant floor(-4.2); // -5
chant ceil(-4.8); // -4
```

A whole number stays as it is. These spells also leave infinities and NaN as they are.

## Native methods

Some native spells belong to a weave. Cast them through a value of that weave:

```eira
bind name = "eira";
chant cast name.to_upper; // EIRA

bind potions = [1, 2];
cast potions.push with 3;
chant potions.size(); // 3
```

The tables below show the reagents you supply. The value before the dot is the
receiver; you don't pass it again!

### Num methods

All of these release a Num. They return a result without changing the original number.

| Method | Reagents | What it does |
| --- | --- | --- |
| `floor` | None | Rounds down, like the standalone spell |
| `ceil` | None | Rounds up, like the standalone spell |
| `abs` | None | Gives the absolute value |
| `round` | None | Rounds to the nearest whole number; halfway values go away from zero |
| `sqrt` | None | Gives the square root; negative numbers give NaN |
| `min` | Other: `Num` | Gives the smaller number |
| `max` | Other: `Num` | Gives the larger number |
| `clamp` | Minimum: `Num`, maximum: `Num` | Keeps the result within these bounds |

For `clamp`, the minimum must not exceed the maximum, and neither bound may be
NaN. The current implementation panics on invalid bounds.

### Text methods

Text methods release a result without changing the original Text. Even `clear`
just releases an empty Text, so save its result if that's what you need!

| Method | Reagents | Release weave | What it does |
| --- | --- | --- | --- |
| `size` | None | `Num` | Counts Unicode scalar values, rather than bytes |
| `is_empty` | None | `Truth` | Checks whether the text is empty |
| `extract` | Index: `Num` | `Text` | Gets the character at a zero-based index |
| `infuse` | Other: `Text` | `Text` | Joins the two texts |
| `to_upper` | None | `Text` | Converts to uppercase |
| `to_lower` | None | `Text` | Converts to lowercase |
| `trim` | None | `Text` | Removes surrounding whitespace |
| `contains` | Part: `Text` | `Truth` | Checks whether the text contains that part |
| `clear` | None | `Text` | Releases empty Text |

NB: `extract` currently declares a Text release but gives an empty value for a
negative or out-of-range index. Use a valid whole-number index. Its character
positions, like `size`, count Unicode scalar values; a displayed symbol can
contain more than one of those.

### Deck methods

Here `W` is the deck's element weave. `Maybe<W>` means the result might be empty.

| Method | Reagents | Release weave | What it does |
| --- | --- | --- | --- |
| `push` | Value: `W` | `Empty` | Adds a value at the end |
| `size` | None | `Num` | Counts the elements |
| `max_size` | None | `Num` | Gives the capacity, or zero for an unbounded deck |
| `pop` | None | `Maybe<W>` | Removes and releases the last element, or empty if there is none |
| `clear` | None | `Empty` | Removes all elements |
| `is_empty` | None | `Truth` | Checks whether the deck is empty |
| `first` | None | `Maybe<W>` | Gets the first element without removing it |
| `last` | None | `Maybe<W>` | Gets the last element without removing it |
| `reverse` | None | `Empty` | Reverses the elements in place |

`push`, `pop`, `clear` and `reverse` change the deck itself, even when its binding
uses `bind`. Pushing into a full fixed-capacity deck causes a runtime error.
`first` and `last` release empty when the deck has no elements.

### Vault methods

For a `Vault<K, V>`, `K` is the key weave and `V` is the value weave.

| Method | Reagents | Release weave | What it does |
| --- | --- | --- | --- |
| `size` | None | `Num` | Counts the entries |
| `contains` | Key: `K` | `Truth` | Checks whether the key exists |
| `get` | Key: `K` | `Maybe<V>` | Gets the value, or empty if the key is missing |
| `remove` | Key: `K` | `Maybe<V>` | Removes and releases the value, or empty if the key is missing |
| `insert` | Key: `K`, value: `V` | `Empty` | Adds an entry or replaces its value |
| `clear` | None | `Empty` | Removes all entries |
| `keys` | None | `Deck<K>` | Releases a deck of the keys |
| `values` | None | `Deck<V>` | Releases a deck of the values |

`insert`, `remove` and `clear` change the vault itself. Adding a new key to a full
fixed-capacity vault causes a runtime error; replacing an existing value is fine.
See [Vaults](vaults.md) for examples and the rules for keys.

Want to see how native spells work in Rust? Peek into [Native spell internals](native-spells.md).
