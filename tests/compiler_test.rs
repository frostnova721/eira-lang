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
fn oversized_deck_reports_one_error_at_its_opening_bracket() {
    let elements = vec!["1"; 300].join(", ");
    let project = SourceProject::new(&format!("bind xs: Deck<Num> = [{elements}];"));
    let mut compiler = project.compiler();
    let error = compiler.compile_to_bytecode().err().expect("reject big deck");
    assert!(error.msg.contains("exceeds the maximum of 255"), "{}", error.msg);
    assert_eq!(compiler.augury.curses.len(), 1, "{}", error.msg);
    // Columns point just past a token, so the '[' at column 22 reports 23.
    let location = &compiler.augury.curses[0].location;
    assert_eq!((location.line, location.column), (1, 23));
}

#[test]
fn failed_expressions_do_not_cascade_into_more_errors() {
    // Each line has exactly one real mistake: an undefined name.
    let project = SourceProject::new(
        "chant missing + 1;\n\
         mark x = missing;\n\
         chant x * 2;\n\
         chant cast nope with 1, x;\n\
         bind xs: Deck<Num> = [1, missing, 3];\n\
         chant xs[0];\n\
         chant missing.field;\n\
         n ~~ missing { chant n + 1; }\n",
    );
    let mut compiler = project.compiler();
    let error = compiler.compile_to_bytecode().err().expect("reject source");
    let lines: Vec<usize> = compiler.augury.curses.iter().map(|c| c.location.line).collect();
    assert_eq!(lines, vec![1, 2, 4, 5, 7, 8], "{}", error.msg);
    assert!(
        compiler.augury.curses.iter().all(|c| c.message.contains("was undefined")),
        "{}",
        error.msg
    );
}

#[test]
fn codegen_errors_are_reported_through_augury() {
    // One expression needing more than 255 live registers. It nests 300 levels deep,
    // so run it with more stack than a test thread gets by default.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let terms = vec!["1"; 300].join(" + ");
            let project = SourceProject::new(&format!("chant 0;\nchant {terms};"));
            let mut compiler = project.compiler();
            let error = compiler
                .compile_to_bytecode()
                .err()
                .expect("reject register overflow");
            assert!(error.msg.contains("Maximum registers allocated"), "{}", error.msg);
            assert!(error.msg.starts_with("curse: "), "{}", error.msg);

            assert_eq!(compiler.augury.curses.len(), 1);
            let diagnostic = &compiler.augury.curses[0];
            assert_eq!(diagnostic.phase, CompilationPhase::CodeGen);
            assert_eq!(diagnostic.location.file, project.root.join("main.eira"));
            assert_eq!(diagnostic.location.line, 2);
            assert!(diagnostic.location.column > 0);
        })
        .unwrap()
        .join()
        .unwrap();
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
        let mut compiler = project.compiler();
        let error = compiler.compile_to_bytecode().err().expect(argument);
        // The argument is checked against the reagent's weave and reports once;
        // the cast does not add a second error about the same argument.
        assert!(
            error.msg.contains("does not match the expected weave 'Num'"),
            "{}",
            error.msg
        );
        assert_eq!(compiler.augury.curses.len(), 1, "{}", error.msg);
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
