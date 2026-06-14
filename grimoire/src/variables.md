# Variables

Variable, think of it as a container holding a value!

To maintain strict type safety across the realms, Eira splits variables into two distinct types: Marks (mutable) and Binds (immutable).

## Marks: Mutable Variables

The mutable variables in Eira are called marks and they are declared with **mark** keyword.

These are the variable which can have their value changed after assignment. Although in Eira, for having a good type safety the mutable variable can only be reassigned with values of same Weave.

consider the code for declaration example.

```eira
// The weave is auto-inferred as Num
mark stamina = 10;

stamina = 5; // ✅ Valid: 5 is a Num
stamina = "Exhausted"; // ❌ Compile Error: Cannot assign Text to a Num Weave
```

## Binds: Immutable variables

> _bind, yes bind a value to a variable! so tight that it won't change it!_

The binds are the immutable variables in Eira. They are declared with the **bind** keyword.

As these are immutables, it cannot be reassigned with a different value, even if of same weave!

```eira
// Declaration of a bind 
bind coffee_monster = "I love coffee!";

// Will result in a compile error since bind values can't be reassigned
// (and coffee monster loves coffee)
coffee_monster = "I hate coffee"; 
```

