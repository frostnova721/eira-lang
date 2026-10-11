use eira::{
    EiraVM, Value,
    compiler::compiler::{CompiledCode, Compiler, CompilerOptions},
    runtime::vm::InterpretResult,
};

fn compile(source: &str) -> Result<CompiledCode, String> {
    Compiler::new(
        "REPL".into(),
        CompilerOptions {
            print_tokens: false,
            print_ast: None,
            print_woven_ast: None,
            print_instructions: false,
            print_bytecode: false,
        },
        None,
    )
    .compile_to_bytecode_from_string(source)
    .map_err(|e| e.msg)
}
fn vm(source: &str) -> EiraVM {
    let mut vm = EiraVM::init(compile(source).unwrap());
    assert!(matches!(vm.start(), InterpretResult::InterpretOk));
    vm
}
fn number(v: Value) -> f64 {
    let Value::Number(n) = v else {
        panic!("expected number: {v:?}")
    };
    n
}

#[test]
fn literal_lookup_missing_and_overwrite() {
    let mut vm = vm(r#"
        mark v: Vault<Text, Num, 2> = vault { "a": 1, "b": 2, };
        spell get():: Maybe<Num> { release v["a"]; }
        spell missing():: Maybe<Num> { release v["missing"]; }
        spell update() { v["a"] = 9; }
        spell size():: Num { release v.size(); }
    "#);
    assert_eq!(number(vm.cast("get", vec![]).unwrap()), 1.0);
    assert!(vm.cast("missing", vec![]).unwrap().is_emptiness());
    vm.cast("update", vec![]).unwrap();
    assert_eq!(number(vm.cast("get", vec![]).unwrap()), 9.0);
    assert_eq!(number(vm.cast("size", vec![]).unwrap()), 2.0);
}

#[test]
fn capacity_failure_is_atomic_and_removal_frees_space() {
    let mut vm = vm(r#"
        mark v: Vault<Num, Text, 1> = vault { 1: "first" };
        spell overflow() { v[2] = "second"; }
        spell old():: Maybe<Text> { release v[1]; }
        spell missing():: Maybe<Text> { release v[2]; }
        spell remove():: Maybe<Text> { release v.remove(1); }
        spell refill() { v.insert(2, "second"); }
        spell contains():: Truth { release v.contains(2); }
    "#);
    assert!(vm.cast("overflow", vec![]).is_err());
    assert_eq!(
        vm.cast("old", vec![]).unwrap(),
        Value::from("first".to_string())
    );
    assert!(vm.cast("missing", vec![]).unwrap().is_emptiness());
    assert_eq!(
        vm.cast("remove", vec![]).unwrap(),
        Value::from("first".to_string())
    );
    vm.cast("refill", vec![]).unwrap();
    assert_eq!(vm.cast("contains", vec![]).unwrap(), Value::Bool(true));
}

#[test]
fn numeric_zero_duplicate_keys_and_fresh_instances() {
    let mut vm = vm(r#"
        spell run():: Num {
            mark v: Vault<Num, Num, 1> = vault { 0: 1, 0: 2 };
            v[-0] = 3;
            release v.size();
        }
    "#);
    for _ in 0..3 {
        assert_eq!(number(vm.cast("run", vec![]).unwrap()), 1.0);
    }
}

#[test]
fn methods_boolean_keys_and_snapshots() {
    let mut vm = vm(r#"
        mark v = vault { true: 7, false: 8 };
        spell run():: Num {
            bind keys = v.keys();
            bind values = v.values();
            v.clear();
            release keys.size() + values.size() + v.size();
        }
    "#);
    assert_eq!(number(vm.cast("run", vec![]).unwrap()), 4.0);
}

#[test]
fn nested_vaults_arguments_returns_and_shared_aliases() {
    let mut vm = vm(r#"
        spell make():: Vault<Text, Vault<Text, Num, 2>, 1> {
            release vault { "inner": vault { "a": 1 } };
        }
        spell size(v: Vault<Text, Num, 2>):: Num { release v.size(); }
        mark v: Vault<Text, Num, 2> = vault {};
        bind alias = v;
        spell run():: Num {
            alias["x"] = 4;
            release size(v);
        }
    "#);
    assert!(matches!(vm.cast("make", vec![]).unwrap(), Value::Vault(_)));
    assert_eq!(number(vm.cast("run", vec![]).unwrap()), 1.0);
}

#[test]
fn zero_capacity_and_unbounded_growth() {
    let mut vm = vm(r#"
        mark empty: Vault<Text, Num, 0> = vault {};
        mark v: Vault<Num, Num> = vault {};
        spell forbidden() { empty["x"] = 1; }
        spell fill():: Num {
            mark n = 0;
            while n < 1000 { v[n] = n * 2; n = n + 1; }
            release v.size();
        }
    "#);
    assert!(vm.cast("forbidden", vec![]).is_err());
    assert_eq!(number(vm.cast("fill", vec![]).unwrap()), 1000.0);
}

#[test]
fn invalid_types_limits_and_literals_are_rejected() {
    for source in [
        "mark v = vault {};",
        "mark v: Vault = vault {};",
        "mark v: Vault<Text> = vault {};",
        "mark v: Vault<Text, Num, -1> = vault {};",
        "mark v: Vault<Text, Num, 1.5> = vault {};",
        "mark v: Vault<Text, Num, 4294967296> = vault {};",
        "mark v: Vault<Deck<Num>, Num> = vault {};",
        "mark v: Vault<Text, Maybe<Num>> = vault {};",
        "mark v = vault { 1: 1, true: 2 };",
        "mark v = vault { 1: 1, 2: true };",
        "mark v: Vault<Text, Num, 1> = vault { \"a\": 1, \"b\": 2 };",
        "mark v = vault { 1: 1 }; v[true] = 2;",
        "mark v = vault { 1: 1 }; v[2] = true;",
        "mark v = vault { 1: 1 }; bind x = v[true];",
        "mark v = vault { 1: 1 }; v.insert(1);",
        "mark v = vault { 1: 1 }; v.insert(1, 2, 3);",
        "mark v = vault { 1: 1 }; bind bad = true; v.insert(bad, 2);",
        "mark v = vault { 1: 1 }; bind bad = true; v.insert(2, bad);",
        "mark v: Vault<Num, Num, 1> = vault {}; mark other: Vault<Num, Num, 2> = v;",
    ] {
        assert!(compile(source).is_err(), "unexpectedly accepted {source}");
    }
}

#[test]
fn literal_in_branches_and_large_literal_register_reuse() {
    let entries = (0..300)
        .map(|n| format!("{n}: {n}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "spell run(flag: Truth):: Num {{ fate flag {{ bind v = vault {{ {entries} }}; release v.size(); }} divert {{ bind v = vault {{ 0: 1 }}; release v.size(); }} }}"
    );
    let mut vm = vm(&source);
    assert_eq!(
        number(vm.cast("run", vec![Value::Bool(true)]).unwrap()),
        300.0
    );
    assert_eq!(
        number(vm.cast("run", vec![Value::Bool(false)]).unwrap()),
        1.0
    );
}

#[test]
fn nan_is_rejected_and_zero_hashes_identically() {
    use eira::values::vault::VaultObject;
    let v = VaultObject::new(Some(1), 0).unwrap();
    v.insert(&Value::Number(-0.0), Value::Number(1.0)).unwrap();
    v.insert(&Value::Number(0.0), Value::Number(2.0)).unwrap();
    assert_eq!(number(v.get(&Value::Number(-0.0)).unwrap()), 2.0);
    assert!(
        v.insert(&Value::Number(f64::NAN), Value::Number(3.0))
            .is_err()
    );
    assert!(v.get(&Value::Number(f64::NAN)).is_err());
    assert_eq!(v.len(), 1);
}

#[test]
fn contextual_vaults_in_records_and_decks() {
    let mut vm = vm(r#"
        sign Store { data: Vault<Text, Num, 1>, }
        bind store = ~Store with { data: vault {}, };
        spell run():: Num {
            store.data = vault { "a": 1 };
            bind deck: Deck<Vault<Text, Num, 1>> = [vault {}];
            release store.data.size() + deck.size();
        }
    "#);
    assert_eq!(number(vm.cast("run", vec![]).unwrap()), 2.0);
}

#[test]
fn literal_capacity_error_is_semantic() {
    let err = compile(r#"bind v: Vault<Text, Num, 1> = vault { "a": 1, "b": 2 };"#).unwrap_err();
    assert!(err.contains("Vault literal exceeds"), "{err}");
}

#[test]
fn expression_keys_are_evaluated_before_values() {
    let mut vm = vm(r#"
        spell run():: Maybe<Num> {
            mark key = 1;
            bind v = vault { key: (key = 2) };
            release v[1];
        }
    "#);
    assert_eq!(number(vm.cast("run", vec![]).unwrap()), 2.0);
}
