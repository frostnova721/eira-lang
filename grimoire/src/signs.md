# Signs

The **Sign** represents the inter-weavings of a material. It is a concept, not a real element!

## Declaration

Signs can be declared simply by `sign` keyword, followed by its name and the marks it contains!

```eira
sign Sword {
    type: Text,
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

The materials of a sign can be defined to have behaviours. These are called **Attunements** and they are defined with the `attune` keyword.
Within these attunements, you can access the material's internal elements using `ego`, which represents the current material instance.

```eira
attune Sword {
    spell cut() {
        chant "It cut an apple!";
    }

    spell material() {
        chant "Made of: ";
        chant ego.type;
    }
}
```

To call an attunement, you use the `cast` keyword with the material and the spell name, similar to casting a spell:

```eira
mark hero_sword = ~Sword with {
    type: "Phoenix Steel",
};

// Calling the attunements!
cast hero_sword.cut;
cast hero_sword.material;
```

This ensures that your materials are not just lifeless data, but have their own unique, magical behaviors!
