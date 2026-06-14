# Signs

The **Sign** represents the inter-weavings of a material. It is a concept, not a real element!

## Declaration

Signs can be declared simply by `sign` keyword, followed by its name and the marks it contains!

```eira
sign Sword {
    type: String,
}
```

## Drawing The Signs (materialization)

The signs can only be used after drawing them (similar to spells). After a sign is drawn, it solidifies into a 'material'.
It is performed as per the following snippet.

```eira
bind hero_sword = ~Sword with {
    type: "Pheonix Steel"
};
```

## Access and Modification

The elements inside the material can be accessed and changed after the sign is drawn.

The following snippet shows that.

```eira
// Access is done like this
chant hero_sword.type; // prints "Pheonix Steel"

// reassigning the type
hero_sword.type = "Stainless Steel" // what a downgrade

chant hero_sword.type; // prints "Stainless Steel"
```

## Attunements

The materials of a sign can be defined to have behaviours. These are called **Attunements** and they are defined with `attune` keyword.

```eira
attune Sword {
    spell cut() {
        chant "It cut an apple!";
    }
}
```
