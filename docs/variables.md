# Variables

Variable, think of it as a container holding a value!

In Eira, we have mutable and immutable variables. They are first class items in Eira!

## Marks: Mutable Variables

The mutable variables in Eira are called marks and they are declared with **mark** keyword.

These are the variable which can have their value changed after assignment. Although in Eira, for having a good type safety the mutable variable can only be assigned with values which are of same weave as the declared one.

consider the code for declaration example.

```eira
// here type is auto inferred to be Num
mark a = 10;

// This will compile & run. 5 is a Num
a = 5;

// this will cause an error, since a is a Num
a = "Hello Eira!"; 
```

## Binds: Immutable variables

> _bind, yes bind a value to a variable! so tight that it won't change it!_

The binds are the immutable variables in Eira. They are declared with the **bind** keyword.

As these are immutables, it cannot be reassigned with a different value, even if of same weave!

```eira
// Declaration of a bind 
bind coffee_monster = "I love coffee!";

// Will result in an error since bind values can't be changed
coffee_monster = "I hate coffee"; 
```

