mod common;

use common::SourceProject;
use std::process::Command;

fn run(project: &SourceProject, args: &[&str]) -> (String, String) {
    // Check compilation independently: the CLI does not yet signal errors with its exit code.
    if let Err(error) = project.compiler().compile_to_bytecode() {
        panic!("test program did not compile: {}", error.msg);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_eira"))
        .current_dir(&project.root)
        .arg(project.root.join("main.eira"))
        .args(args)
        .output()
        .expect("run Eira");
    assert!(output.status.success(), "{output:?}");
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn assert_output(source: &str, expected: &str) {
    let (stdout, stderr) = run(&SourceProject::new(source), &[]);
    assert_eq!(stderr, "");
    assert_eq!(stdout, expected);
}

#[test]
fn arithmetic_precedence_text_and_truth() {
    assert_output(
        "chant 2 + 3 * 4; chant (2 + 3) * 4; chant -5; chant 7 % 3; chant 9 / 3; chant \"hello\" + \" world\"; chant !false; chant 3 <= 3;",
        "14\n20\n-5\n1\n3\nhello world\ntrue\ntrue\n",
    );
}

#[test]
fn loops_branches_continue_and_break() {
    assert_output(
        r#"
        mark i = 0;
        mark total = 0;
        while i < 10 {
            i = i + 1;
            fate i == 2 { flow; }
            fate i == 5 { sever; }
            total = total + i;
        }
        fate total == 8 { chant "ok"; } divert { chant "wrong"; }
        fate false { chant "wrong"; } divert { chant i; }
        "#,
        "ok\n5\n",
    );
}

#[test]
fn recursive_spell() {
    assert_output(include_str!("../eira_scripts/factorial.eira"), "120\n");
}

#[test]
#[ignore = "Known bug: returned closure inherits outer spell reagent metadata"]
fn returned_closure_keeps_captured_parameter() {
    assert_output(
        r#"
        spell outer(n: Num):: Spell<Num> {
            spell inner():: Num { release n + 1; }
            release inner;
        }
        mark first = cast outer with 41;
        mark second = cast outer with 9;
        chant cast first;
        chant cast second;
        chant cast first;
        "#,
        "42\n10\n42\n",
    );
}

#[test]
fn conditional_release_and_implicit_return() {
    assert_output(
        r#"
        spell check(n: Num) {
            fate n <= 1 { chant "early"; release; }
        }
        cast check with 10;
        chant "returned";
        cast check with 1;
        chant "done";
        "#,
        "returned\nearly\ndone\n",
    );
}

#[test]
fn struct_methods_can_access_receiver() {
    assert_output(
        include_str!("../eira_scripts/test_attunement.eira"),
        "It cut an apple!\nMade of: \nPhoenix Steel\nIt cut an apple!\nMade of: \nPhoenix Steel\n",
    );
}

#[test]
fn struct_field_assignment() {
    assert_output(
        "sign Box { value: Num, } mark box = ~Box with { value: 1, }; box.value = 7; chant box.value;",
        "7\n",
    );
}

#[test]
fn dynamic_deck_mutation_and_append() {
    assert_output(
        "bind xs: Deck<Num> = [1, 2]; xs[0] = xs[0] + 9; xs[2] = 30; chant xs[0]; chant xs[1]; chant xs[2];",
        "10\n2\n30\n",
    );
}

#[test]
fn fixed_deck_capacity_error_stops_execution() {
    let project =
        SourceProject::new("bind xs: Deck<Num, 1> = [1]; xs[1] = 2; chant \"unreachable\";");
    let (stdout, stderr) = run(&project, &[]);
    let diagnostic = format!("{stdout}{stderr}");
    assert!(diagnostic.contains("Index out of bounds"), "{diagnostic}");
    assert!(!diagnostic.contains("unreachable"), "{diagnostic}");
}

#[test]
fn path_import_with_namespace() {
    let project = SourceProject::new("tether \"library.eira\" bind lib; chant cast lib.answer;");
    project.write("library.eira", "forge spell answer():: Num { release 42; }");
    let (stdout, stderr) = run(&project, &[]);
    assert_eq!(stderr, "");
    assert_eq!(stdout, "42\n");
}

#[test]
fn no_run_compiles_without_executing() {
    let project = SourceProject::new("chant \"should not run\";");
    let (stdout, stderr) = run(&project, &["--no-run"]);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
}

#[test]
fn nested_closure_reads_captured_local() {
    assert_output(
        "spell outer() { mark n = 41; spell inner() { chant n + 1; } cast inner; } cast outer;",
        "42\n",
    );
}

#[test]
#[ignore = "Known bug: bare source filename resolves sibling imports from filesystem root"]
fn relative_cli_source_resolves_sibling_import() {
    let project = SourceProject::new("tether \"library.eira\"; chant cast answer;");
    project.write("library.eira", "forge spell answer():: Num { release 42; }");
    let output = Command::new(env!("CARGO_BIN_EXE_eira"))
        .current_dir(&project.root)
        .arg("main.eira")
        .output()
        .expect("run Eira");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
}

#[test]
fn iterator_example_traverses_deck_and_inclusive_range() {
    assert_output(
        include_str!("../eira_scripts/iterators.eira"),
        "ember\nfrost\nstorm\n1\n2\n3\n",
    );
}

#[test]
fn cycle_over_empty_deck_skips_body() {
    assert_output("item ~~ [] { chant item; } chant \"done\";", "done\n");
}

#[test]
fn cycle_range_boundaries() {
    assert_output(
        "n ~~ 3..3 { chant n; } n ~~ 3..1 { chant n; } chant \"done\";",
        "3\ndone\n",
    );
}

#[test]
fn cycle_accepts_bound_deck_and_accumulates_values() {
    assert_output(
        "bind values = [2, 4, 6]; mark total = 0; value ~~ values { total = total + value; } chant total;",
        "12\n",
    );
}

#[test]
#[ignore = "Known bug: nested cycles overwrite the outer iteration value and panic in the VM"]
fn nested_cycles_visit_every_pair() {
    assert_output(
        "a ~~ 1..2 { b ~~ [3, 4] { chant a * 10 + b; } }",
        "13\n14\n23\n24\n",
    );
}
