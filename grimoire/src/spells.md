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
