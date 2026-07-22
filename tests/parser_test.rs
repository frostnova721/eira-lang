#[cfg(test)]
mod parser_test {
    use eira::{
        Parser, Scanner, Value,
        compiler::{Expr, Stmt, token_type::TokenType, ast::decl::Decl},
    };

    fn parse_helper(source: &str) -> Vec<Decl> {
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();
        let parser = Parser::new(tokens, "parser_test".to_string());
        parser.parse().unwrap()
    }

    #[test]
    fn test_variable_declaration() {
        let src = r#"
            mark a = 69;
            bind b = "meow";
        "#;

        let statements = parse_helper(src);

        assert_eq!(statements.len(), 2);

        // check n verify the mark
        if let Decl::VarDeclaration {
            name,
            mutable,
            initializer,
            visibility: _,
            weave: _,
        } = &statements[0]
        {
            assert_eq!(name.lexeme, "a");
            assert_eq!(*mutable, true);
            assert!(initializer.is_some());
        } else {
            panic!("Expected VarDeclaration for 'bind'");
        }

        // check n verify the bind
        if let Decl::VarDeclaration {
            name,
            mutable,

            initializer,
            visibility: _,
            weave: _,
        } = &statements[1]
        {
            assert_eq!(name.lexeme, "b");
            assert_eq!(*mutable, false);
            assert!(initializer.is_some());
        } else {
            panic!("Expected VarDeclaration for 'bind'");
        }
    }

    #[test]
    fn test_precedence() {
        // Should parse as: (-1) + (2 * 3)
        let source = "-1 + 2 * 3;";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        // fragile check, but OK
        if let Decl::Statement { stmt: box_stmt, token: _ } = &stmts[0] {
            if let Stmt::ExprStmt { expr } = &**box_stmt {
                if let Expr::Binary {
                    left,
                    operator,
                    right,
                } = expr
                {
                    assert_eq!(operator.token_type, TokenType::Plus);

                    // Check left side is Unary
                    assert!(matches!(**left, Expr::Unary { .. }));

                    // Check right side is Binary
                    assert!(matches!(**right, Expr::Binary { .. }));
                } else {
                    panic!("Expected top-level expression to be Binary");
                }
            } else {
                panic!("Expected Expression Statement");
            }
        } else {
            panic!("Expected Statement Declaration wrapper");
        }
    }

    #[test]
    fn test_grouping() {
        // Should parse as: (1 + 2) * 3
        let source = "(1 + 2) * 3;";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        if let Decl::Statement { stmt: box_stmt, token: _ } = &stmts[0] {
            if let Stmt::ExprStmt { expr } = &**box_stmt {
                if let Expr::Binary {
                    left,
                    operator,
                    right,
                } = expr
                {
                    assert_eq!(operator.token_type, TokenType::Star);
                    assert!(matches!(**left, Expr::Grouping { .. }));
                    assert!(matches!(**right, Expr::Literal { .. }));
                } else {
                    panic!("Expected top-level expression to be Binary");
                }
            } else {
                panic!("Expected Expression Statement");
            }
        } else {
            panic!("Expected Statement Declaration wrapper");
        }
    }

    #[test]
    fn test_fate_statement() {
        let source = "fate true { chant 1; } divert { chant 2; }";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        if let Decl::Statement { stmt: box_stmt, token: _ } = &stmts[0] {
            if let Stmt::Fate {
                condition,
                then_branch,
                else_branch,
            } = &**box_stmt
            {
                assert!(matches!(
                    condition,
                    Expr::Literal {
                        value: Value::Bool(true),
                        token: _
                    }
                ));
                assert!(matches!(**then_branch, Stmt::Block { .. }));
                assert!(else_branch.is_some());
                if let Some(else_b) = else_branch {
                    assert!(matches!(**else_b, Stmt::Block { .. }));
                }
            } else {
                panic!("Expected a Fate (if) statement.");
            }
        } else {
            panic!("Expected Statement Declaration wrapper");
        }
    }

    #[test]
    fn test_while_statement() {
        let source = "while x < 5 { chant x; }";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        if let Decl::Statement { stmt: box_stmt, token: _ } = &stmts[0] {
            if let Stmt::While { condition, body } = &**box_stmt {
                assert!(matches!(condition, Expr::Binary { .. }));
                assert!(matches!(**body, Stmt::Block { .. }));
            } else {
                panic!("Expected a While statement.");
            }
        } else {
            panic!("Expected Statement Declaration wrapper");
        }
    }
}
