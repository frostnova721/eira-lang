# The Grimoire of Eira

## Compile phases

Eira's compiler is comprised of 5 stages, with 3 passes till the final Eira bytecode (remnants).

```eira
( Scanner ) ⟶ ( Parser ) ⟶ ( Weave Analyzer ) ⟶ ( Code Generation )
```

### Scan Phase

The Scanner picks up all the raw eira code and constructs the respective tokens for the keywords and expressions.

### Parse Phase

This is the phase where the **Tokens** produced by the scanner is made sense. The tokens will be arranged according to the rules of Eira!

### Weave Analyze Phase

This is where the Eira's Weave system is checked. The behavioural capabilities of the variables, expressions are all checked & verified in this phase. This phase also resolves the variables and their scoping.

### Code Gen Phase

The Code Gen generates the Eira bytecode from the resulting AST

**currently whipping up the outer world's mana for the spells! Stay tuned for more!**
