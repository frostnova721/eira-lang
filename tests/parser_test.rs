#[cfg(test)]
mod parser_test {
    use eira::{
        Parser, Scanner, Value,
        frontend::{expr::Expr, stmt::Stmt, token_type::TokenType},
    };

    fn parse_helper(source: &str) -> Vec<Stmt> {
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();
        let parser = Parser::new(tokens);
        parser.parse()
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
        if let Stmt::VarDeclaration {
            name,
            mutable,
            initializer,
        } = &statements[0]
        {
            assert_eq!(name.lexeme, "a");
            assert_eq!(*mutable, true);
            assert!(initializer.is_some());
        } else {
            panic!("Expected VarDeclaration for 'bind'");
        }

        // check n verify the bind
        if let Stmt::VarDeclaration {
            name,
            mutable,
            initializer,
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
        if let Stmt::ExprStmt { expr } = &stmts[0] {
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
    }

    #[test]
    fn test_grouping() {
        // Should parse as: (1 + 2) * 3
        let source = "(1 + 2) * 3;";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        if let Stmt::ExprStmt { expr } = &stmts[0] {
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
    }

    #[test]
    fn test_fate_statement() {
        let source = "fate true { chant 1; } divert { chant 2; }";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        if let Stmt::Fate { condition, then_branch, else_branch } = &stmts[0] {
            assert!(matches!(condition, Expr::Literal { value: Value::Bool(true) }));
            assert!(matches!(**then_branch, Stmt::Block { .. }));
            assert!(else_branch.is_some());
            if let Some(else_b) = else_branch {
                assert!(matches!(**else_b, Stmt::Block { .. }));
            }
        } else {
            panic!("Expected a Fate (if) statement.");
        }
    }

    #[test]
    fn test_while_statement() {
        let source = "while x < 5 { chant x; }";
        let stmts = parse_helper(source);
        assert_eq!(stmts.len(), 1);

        if let Stmt::While { condition, body } = &stmts[0] {
            assert!(matches!(condition, Expr::Binary{..}));
            assert!(matches!(**body, Stmt::Block{..}));
        } else {
            panic!("Expected a While statement.");
        }
    }
}
