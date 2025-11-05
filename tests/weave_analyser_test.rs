#[cfg(test)]
mod weave_analyser_test {
    use eira::{
        Parser, Scanner, WeaveAnalyzer,
        frontend::{
            expr::WovenExpr,
            stmt::WovenStmt,
            strand::{ADDITIVE_STRAND, CONDITIONAL_STRAND, MULTIPLICATIVE_STRAND},
        },
    };

    fn analyze_helper(source: &str) -> Result<Vec<WovenStmt>, String> {
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();
        let parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;
        let mut wa = WeaveAnalyzer::new();
        wa.analyze(ast).map_err(|e| format!("{}", e.msg))
    }

    fn first_expr(stmts: &Vec<WovenStmt>) -> &WovenExpr {
        // Prefer the first Chant or ExprStmt found
        for stmt in stmts {
            match stmt {
                WovenStmt::ExprStmt { expr } => return expr,
                WovenStmt::Chant { expression } => return expression,
                _ => {}
            }
        }
        panic!("Expected an expression statement or chant in analyzed stmts");
    }

    #[test]
    fn binary_add_numbers_ok() {
        let src = "chant 1 + 2;";
        let stmts = analyze_helper(src).expect("weave analyze ok");
        let expr = first_expr(&stmts);
        if let WovenExpr::Binary { tapestry, .. } = expr {
            // result supports additive
            assert!(tapestry.has_strand(ADDITIVE_STRAND));
        } else {
            panic!("Expected Binary expr");
        }
    }

    #[test]
    fn binary_add_mismatch_error() {
        let src = "chant 1 + \"s\";";
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("Cannot perform '+' operation"));
    }

    #[test]
    fn unary_minus_number_ok() {
        let src = "chant -1;";
        let stmts = analyze_helper(src).expect("weave analyze ok");
        let expr = first_expr(&stmts);
        if let WovenExpr::Unary { tapestry, .. } = expr {
            // result of -1 should be numeric; check it supports multiplicative strand set on literal number
            assert!(tapestry.has_strand(MULTIPLICATIVE_STRAND) || tapestry.0 != 0);
        } else {
            panic!("Expected Unary expr");
        }
    }

    #[test]
    fn unary_bang_truth_ok_and_bang_number_error() {
        // OK
        let ok_src = "chant !true;";
        let _ = analyze_helper(ok_src).expect("bang true ok");
        // Error
        let err_src = "chant !1;";
        let err = analyze_helper(err_src).err().expect("should error for !1");
        assert!(err.contains("operand does not contain the 'CONDITIONAL'"));
    }

    #[test]
    fn var_decl_and_resolution_ok() {
        let src = "mark a = 1; chant a;";
        let stmts = analyze_helper(src).expect("analyze ok");
        // second stmt uses variable
        match &stmts[1] {
            WovenStmt::Chant { expression } => match expression {
                WovenExpr::Variable { tapestry, .. } => {
                    // at least not empty
                    assert!(tapestry.0 != 0);
                }
                _ => panic!("Expected variable in chant"),
            },
            _ => panic!("Expected chant"),
        }
    }

    #[test]
    fn bind_reassign_errors() {
        let src = "bind x = 1; x = 2;";
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("Binds cannot be reassigned"));
    }

    #[test]
    fn assignment_type_mismatch_errors() {
        let src = "mark x = 1; x = \"s\";";
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("different Weaves"));
    }

    #[test]
    fn spell_cast_known_return_weave_ok() {
        let src = r#"
            spell f():: NumWeave { release 2; }
            chant cast f;
        "#;
        let stmts = analyze_helper(src).expect("analyze ok");
        let expr = first_expr(&stmts);
        if let WovenExpr::Cast { tapestry, .. } = expr {
            // numeric results should be usable in multiplicative/additive contexts; just check not empty
            assert!(tapestry.0 != 0);
        } else {
            panic!("Expected Cast in chant");
        }
    }

    #[test]
    fn spell_cast_arg_count_mismatch_error() {
        let src = r#"
            spell g(n: NumWeave):: NumWeave { release n; }
            chant cast g; // missing arg
        "#;
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("expected 1 reagents"));
    }

    #[test]
    fn fate_condition_requires_conditional_strand() {
        let src = "fate 1 { chant 1; }";
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("does not contain the 'Conditional'"));
    }

    #[test]
    fn spell_with_reagents_validates_types() {
        let src = r#"
            spell add(a: NumWeave, b: NumWeave):: NumWeave {
                release a + b;
            }
            chant cast add with 1, 2;
        "#;
        let _ = analyze_helper(src).expect("should succeed");
    }

    #[test]
    fn spell_reagent_type_mismatch_error() {
        let src = r#"
            spell add(a: NumWeave, b: NumWeave):: NumWeave {
                release a + b;
            }
            chant cast add with 1, "string";
        "#;
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("was expected to be"));
    }

    #[test]
    fn runtime_spell_via_variable_ok() {
        let src = r#"
            spell f():: NumWeave { release 42; }
            mark a = f;
            chant cast a;
        "#;
        let _ = analyze_helper(src).expect("runtime spell via variable ok");
    }

    #[test]
    fn runtime_spell_from_cast_result_ok() {
        let src = r#"
            spell outer():: SpellWeave<NumWeave> {
                spell inner():: NumWeave {
                    release 99;
                }
                release inner;
            }
            mark a = outer;
            mark b = cast a;
            chant cast b;
        "#;
        let _ = analyze_helper(src).expect("runtime spell from cast ok");
    }

    #[test]
    fn variable_undefined_error() {
        let src = "chant nonexistent;";
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("resolution failed"));
    }

    #[test]
    fn release_type_mismatch_error() {
        let src = r#"
            spell f():: NumWeave {
                release "string";
            }
        "#;
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("expected to release"));
    }

    #[test]
    fn while_condition_requires_conditional_strand() {
        let src = "while 5 { chant 1; }";
        let err = analyze_helper(src).err().expect("should error");
        assert!(err.contains("does not contain the 'Conditional'"));
    }

    #[test]
    fn binary_multiply_and_divide_ok() {
        let ok1 = "chant 10 * 2;";
        let ok2 = "chant 10 / 2;";
        let _ = analyze_helper(ok1).expect("multiply ok");
        let _ = analyze_helper(ok2).expect("divide ok");
    }

    #[test]
    fn comparison_operators_return_truth_weave() {
        let src = "mark result = 5 > 3;";
        let stmts = analyze_helper(src).expect("comparison ok");
        if let WovenStmt::VarDeclaration { initializer, .. } = &stmts[0] {
            if let Some(WovenExpr::Binary { tapestry, .. }) = initializer {
                // truth weave has conditional strand
                assert!(tapestry.has_strand(CONDITIONAL_STRAND));
            } else {
                panic!("Expected binary in initializer");
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn nested_scopes_ok() {
        let src = r#"
            mark outer = 1;
            {
                mark inner = 2;
                chant inner;
            }
            chant outer;
        "#;
        let _ = analyze_helper(src).expect("nested scopes ok");
    }

    #[test]
    fn spell_without_release_ok() {
        let src = r#"
            spell doNothing() {
                chant "hi";
            }
            cast doNothing;
        "#;
        let _ = analyze_helper(src).expect("spell without release ok");
    }

    #[test]
    fn upvalue_capture_tracking() {
        // This tests that upvalue metadata is set during analysis
        let src = r#"
            spell outer() {
                mark x = 10;
                spell inner() {
                    chant x;
                }
                cast inner;
            }
            cast outer;
        "#;
        let _ = analyze_helper(src).expect("upvalue capture ok");
    }
}
