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
fn parse_num_converts_text_to_numbers() {
    assert_output(
        r#"
        mark n: Num = cast parseNum with "42";
        chant n + 1;
        chant parseNum("-12.5");
        chant parseNum("  +3.25  ");
        chant parseNum("1.5e2");
        chant parseNum("0");
        "#,
        "43\n-12.5\n3.25\n150\n0\n",
    );
}

#[test]
fn parse_num_invalid_text_stops_execution() {
    for input in ["", "   ", "hello", "12abc", "1.2.3"] {
        let project = SourceProject::new(&format!(
            "chant parseNum({input:?}); chant \"unreachable\";"
        ));
        let (stdout, stderr) = run(&project, &[]);
        let diagnostic = format!("{stdout}{stderr}");
        assert!(diagnostic.contains("parseNum could not convert"), "{diagnostic}");
        assert!(!diagnostic.contains("unreachable"), "{diagnostic}");
    }
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
fn nested_cycles_visit_every_pair() {
    assert_output(
        "a ~~ 1..2 { b ~~ [3, 4] { chant a * 10 + b; } }",
        "13\n14\n23\n24\n",
    );
}

#[test]
fn registers_are_reused_across_statements() {
    // Each statement used to keep its temporaries forever, overflowing 255 registers.
    let source: String = (1..=300).map(|n| format!("chant {n};\n")).collect();
    let expected: String = (1..=300).map(|n| format!("{n}\n")).collect();
    assert_output(&source, &expected);

    let source = format!(
        "spell many() {{ mark total = 0; {} chant total; }} cast many;",
        "total = total + 1;\n".repeat(300)
    );
    assert_output(&source, "300\n");
}

#[test]
fn spell_body_local_does_not_clobber_parameter() {
    assert_output(
        "spell f(a: Num, b: Num) { mark c = 5; mark d = 6; chant a; chant b; chant c + d; } cast f with 1, 2;",
        "1\n2\n11\n",
    );
}

#[test]
fn nested_block_locals_keep_their_own_registers() {
    assert_output(
        "{ mark x = 1; { mark y = 2; chant x; chant y; } chant x; }",
        "1\n2\n1\n",
    );
}

#[test]
fn cycle_does_not_overwrite_iterated_local() {
    assert_output(
        "{ bind d: Deck<Num> = [1, 2]; x ~~ d { chant x; } chant d[0]; }",
        "1\n2\n1\n",
    );
}

#[test]
fn branches_containing_decks_and_signs_jump_correctly() {
    // NewDeck and NewSign declared the wrong byte size, so jumps over them landed mid-instruction.
    assert_output(
        r#"
        sign Point { x: Num, }
        mark i = 0;
        while i < 2 {
            fate i == 0 {
                bind e: Deck<Num> = [5];
                bind p = ~Point with { x: 7, };
                chant e[0] + p.x;
            }
            i = i + 1;
        }
        chant "done";
        "#,
        "12\ndone\n",
    );
}

#[test]
fn closure_captures_local_of_spell_with_upvalues() {
    // `middle` keeps its own capture in register 0, so `b` is not at its slot index.
    assert_output(
        r#"
        spell outer() {
            mark a = 1;
            spell middle() {
                chant a;
                mark b = 20;
                spell inner() { chant b; }
                cast inner;
            }
            cast middle;
        }
        cast outer;
        "#,
        "1\n20\n",
    );
}

#[test]
fn runtime_error_reports_location_and_trace() {
    let project = SourceProject::new(
        "spell inner(xs: Deck<Num>):: Num {\n    release xs[5];\n}\nchant cast inner with [1, 2];\nchant \"unreachable\";\n",
    );
    let (stdout, stderr) = run(&project, &[]);
    assert!(!stdout.contains("unreachable"), "{stdout}");
    assert!(stderr.contains("Index out of bounds"), "{stderr}");
    assert!(stderr.contains("main.eira:2:"), "{stderr}");
    assert!(stderr.contains("(spell 'inner')"), "{stderr}");
    assert!(stderr.contains("main.eira:4:"), "{stderr}");
}

#[test]
fn negative_and_fractional_deck_indexes_are_errors() {
    for index in ["-1", "1.5"] {
        let project = SourceProject::new(&format!(
            "bind d: Deck<Num> = [1, 2, 3]; chant d[{index}]; chant \"unreachable\";"
        ));
        let (stdout, stderr) = run(&project, &[]);
        assert!(stderr.contains("whole, non-negative"), "{stderr}");
        assert!(!stdout.contains("unreachable"), "{stdout}");
    }
}

#[test]
fn runaway_recursion_is_a_runtime_error() {
    let project = SourceProject::new(
        "spell forever(n: Num):: Num { release cast forever with n + 1; } chant cast forever with 0;",
    );
    let (_, stderr) = run(&project, &[]);
    assert!(stderr.contains("Stack overflow"), "{stderr}");
    assert!(stderr.contains("more times"), "{stderr}");
}
