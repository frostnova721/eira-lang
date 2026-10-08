mod common;

use common::SourceProject;
use eira::compiler::diagnostics::CompilationPhase;

#[test]
fn collects_independent_errors_with_source_locations() {
    let project = SourceProject::new("chant missing;\nchant 1 + \"text\";");
    let mut compiler = project.compiler();
    let error = compiler
        .compile_to_bytecode()
        .err()
        .expect("reject invalid source");
    assert!(error.msg.contains("was undefined"), "{}", error.msg);
    assert!(error.msg.contains("Cannot perform '+'"), "{}", error.msg);
    assert_eq!(compiler.augury.curses.len(), 2);
    for (diagnostic, line) in compiler.augury.curses.iter().zip([1, 2]) {
        assert_eq!(diagnostic.location.file, project.root.join("main.eira"));
        assert_eq!(diagnostic.location.line, line);
        assert!(diagnostic.location.column > 0);
        assert!(matches!(diagnostic.phase, CompilationPhase::Weave));
    }
}

#[test]
fn rejects_invalid_deck_literals_before_codegen() {
    for (source, message) in [
        ("bind xs = [1, \"text\"];", "same weave"),
        ("bind xs: Deck<Num, 1> = [1, 2];", "specified capacity"),
        ("bind xs: Deck<Num> = [\"text\"];", "same weave"),
    ] {
        let project = SourceProject::new(source);
        let error = project
            .compiler()
            .compile_to_bytecode()
            .err()
            .expect(source);
        assert!(error.msg.contains(message), "{source}: {}", error.msg);
    }
}

#[test]
fn rejects_wrong_spell_argument_weaves_for_literals_variables_and_expressions() {
    for argument in ["\"text\"", "text", "1 < 2"] {
        let project = SourceProject::new(&format!(
            "spell identity(n: Num):: Num {{ release n; }} bind text = \"text\"; chant cast identity with {argument};"
        ));
        let error = project
            .compiler()
            .compile_to_bytecode()
            .err()
            .expect(argument);
        assert!(
            error.msg.contains("reagent #1 was expected to be"),
            "{}",
            error.msg
        );
    }
}

#[test]
fn missing_source_is_a_compile_error() {
    let project = SourceProject::new("");
    let mut compiler = project.compiler();
    compiler.source_path = project
        .root
        .join("missing.eira")
        .to_str()
        .unwrap()
        .to_owned();
    assert!(compiler.compile_to_bytecode().is_err());
}

#[test]
fn missing_import_is_a_diagnostic() {
    let project = SourceProject::new("tether \"missing.eira\";");
    let error = project
        .compiler()
        .compile_to_bytecode()
        .err()
        .expect("missing import");
    assert!(error.msg.contains("Failed to read scroll"), "{}", error.msg);
}

#[test]
fn imported_scroll_rejects_top_level_execution() {
    let project = SourceProject::new("tether \"library.eira\";");
    project.write("library.eira", "chant 42;");
    let error = project
        .compiler()
        .compile_to_bytecode()
        .err()
        .expect("invalid import");
    assert!(
        error.msg.contains("Only declarations are allowed"),
        "{}",
        error.msg
    );
}

#[test]
fn cycles_reject_non_iterable_values() {
    for value in ["42", "true", "\"text\""] {
        let project = SourceProject::new(&format!("item ~~ {value} {{ chant item; }}"));
        let error = project.compiler().compile_to_bytecode().err().expect(value);
        assert!(
            error.msg.contains("Cannot iterate over a value"),
            "{}",
            error.msg
        );
    }
}

#[test]
fn cycle_variable_does_not_escape_its_scope() {
    let project = SourceProject::new("item ~~ [1, 2] { chant item; } chant item;");
    let error = project
        .compiler()
        .compile_to_bytecode()
        .err()
        .expect("out-of-scope item");
    assert!(error.msg.contains("was undefined"), "{}", error.msg);
}
