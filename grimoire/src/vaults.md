# Vaults (maps)

`Vault<K, V>` is a built-in hash map; no import is required. `Vault<K, V, N>`
limits the number of distinct entries to N. The limit does not eagerly allocate N
slots and is not a bound on total bytes. Bounds currently range from 0 to
4294967295. Different bounds are different types.

```eira
mark inventory: Vault<Text, Num, 3> = vault {
    "moonstone": 3,
    "ember": 8,
};
inventory["moonstone"] = 4;
bind amount: Maybe<Num> = inventory["moonstone"];
chant inventory.contains("ember");
chant inventory.size();
```

Nonempty literals infer key/value types: `bind names = vault { 1: "Ember" };`.
Empty literals require a contextual type: `mark names: Vault<Num, Text> = vault {};`.
Nested vaults are supported. Entries evaluate left to right, key before value;
repeated keys overwrite their earlier values. Provably excessive literals are
compile errors; dynamic insertions check the limit at runtime.

Keys are `Num`, `Text`, or `Truth`. Numeric zero and negative zero identify the
same entry; NaN is rejected at runtime. Infinities are supported. Strings are
compared by contents. Mutable records and collections cannot be keys.

Lookup returns `Maybe<V>`: a missing key is emptiness. Currently `V` cannot be
`Maybe<T>` or `Empty`, because the runtime cannot distinguish nested optional
states. Other value types, including records, decks, and vaults, are supported.

| Operation | Result |
| --- | --- |
| `v[key]`, `v.get(key)` | `Maybe<V>` |
| `v[key] = value` | Assigned value (`V`) |
| `v.insert(key, value)` | Empty |
| `v.contains(key)` | Truth |
| `v.remove(key)` | Previous value as `Maybe<V>` |
| `v.size()` | Num |
| `v.clear()` | Empty; retains allocated table storage |
| `v.keys()` | Snapshot `Deck<K>` |
| `v.values()` | Snapshot `Deck<V>` |

Replacing a key in a full vault succeeds. Adding a new key to a full vault
reports a runtime error without changing existing entries. Removing or clearing
entries makes space available again. A zero-limit vault must remain empty.

Assignment shares the collection, as with Deck: `bind alias = inventory;` does
not copy its contents. `bind` prevents rebinding, not mutation of the collection.
Snapshots copy the list of keys/values; collection-valued elements remain shared.
Iteration order is unspecified. Iterate `v.keys()` using the existing Deck
iteration syntax; direct Vault iteration is not provided yet.

The implementation uses Rust's randomized HashMap, dedicated get/set bytecode,
reference-counted string keys without copying their bytes, and an entry-based
single-lookup insert/update. Literals reserve storage for their entries (capped
by the bound) and reuse temporary registers. Expected lookup/insertion complexity
is amortized O(1) in entry count; hashing a string still costs O(string length).
