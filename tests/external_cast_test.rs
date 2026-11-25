use eira::{
    EiraVM, Value,
    compiler::compiler::{Compiler, CompilerOptions},
    runtime::vm::InterpretResult,
};

fn initialized_vm(source: &str) -> EiraVM {
    let source_path = "REPL".to_string();
    let options = CompilerOptions {
        print_tokens: false,
        print_ast: None,
        print_woven_ast: None,
        print_instructions: false,
        print_bytecode: false,
    };
    let project = None;
    let mut compiler = Compiler::new(source_path, options, project);
    let compiled = compiler
        .compile_to_bytecode_from_string(source)
        .expect("Host-call fixture should compile");
    let mut vm = EiraVM::init(compiled);
    assert!(matches!(vm.start(), InterpretResult::InterpretOk));
    vm
}

#[test]
fn external_cast_error_carries_location_and_vm_stays_usable() {
    let mut vm = initialized_vm(
        "spell pick(i: Num):: Num {\n    bind xs: Deck<Num> = [10, 20];\n    release xs[i];\n}\n",
    );

    let error = vm.cast("pick", vec![Value::Number(7.0)]).unwrap_err();
    assert!(error.message.contains("Index out of bounds"), "{error}");
    let location = error.location().expect("error should have a location");
    assert_eq!(location.file, "REPL");
    assert_eq!(location.line, 3);
    assert_eq!(error.trace[0].spell.as_deref(), Some("pick"));

    // The failed call is cleaned up, so the next one works.
    assert_eq!(vm.cast("pick", vec![Value::Number(1.0)]).unwrap(), Value::Number(20.0));
}

#[test]
fn external_cast_rejects_unknown_spell_without_trace() {
    let mut vm = initialized_vm("spell add(a: Num, b: Num):: Num { release a + b; }");
    let error = vm.cast("missing", vec![]).unwrap_err();
    assert!(error.message.contains("not found"), "{error}");
    assert!(error.trace.is_empty());
}

#[test]
fn external_cast_add() {
    let mut vm = initialized_vm("spell add(a: Num, b: Num):: Num { release a + b; }");

    let result = vm
        .cast("add", vec![Value::Number(2.0), Value::Number(3.0)])
        .unwrap();

    assert_eq!(result, Value::Number(5.0));

    // Reuse the initialized module without restarting its top-level code.
    let result = vm
        .cast("add", vec![Value::Number(10.0), Value::Number(-4.0)])
        .unwrap();
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn external_cast_returns_after_nested_spell_call() {
    let mut vm = initialized_vm(
        r#"
        spell add(a: Num, b: Num):: Num {
            release a + b;
        }

        spell double_then_increment(n: Num):: Num {
            bind doubled = cast add with n, n;
            release doubled + 1;
        }
        "#,
    );

    let result = vm.cast("double_then_increment", vec![Value::Number(6.0)]).unwrap();
    assert_eq!(result, Value::Number(13.0));
}

#[test]
fn external_cast_calls_attuned_method_through_wrapper() {
    // cast() resolves global spells. The wrapper supplies the method receiver.
    let mut vm = initialized_vm(
        r#"
        sign Counter {
            value: Num,
        }

        attune Counter {
            spell increase(amount: Num):: Num {
                ego.value = ego.value + amount;
                release ego.value;
            }
        }

        bind counter = ~Counter with { value: 10, };

        spell increase_counter(amount: Num):: Num {
            release counter.increase(amount);
        }
        "#,
    );

    let result = vm.cast("increase_counter", vec![Value::Number(5.0)]).unwrap();
    assert_eq!(result, Value::Number(15.0));

    // The same receiver retains its field value across host calls.
    let result = vm.cast("increase_counter", vec![Value::Number(2.0)]).unwrap();
    assert_eq!(result, Value::Number(17.0));
}
