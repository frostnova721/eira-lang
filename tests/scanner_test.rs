#[cfg(test)]
mod scanner_test {
    use eira::{compiler::token_type::TokenType, Scanner};

    #[test]
    fn test_single_character_tokens() {
        let source = "(){};:.,+-/*";
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::ParenLeft,
            TokenType::ParenRight,
            TokenType::BraceLeft,
            TokenType::BraceRight,
            TokenType::SemiColon,
            TokenType::Colon,
            TokenType::Dot,
            TokenType::Comma,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Slash,
            TokenType::Star,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (i, token_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *token_type);
        }
    }

    #[test]
    fn test_multi_character_tokens() {
        let source = "! != = == > >= < <=";
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (i, token_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *token_type);
        }
    }

    #[test]
    fn test_string_literal() {
        let source = r#""hello world""#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        assert_eq!(tokens.len(), 2); 
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].lexeme, "hello world"); 
    }

    #[test]
    fn test_unterminated_string() {
        let source = r#""this is not finished"#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();
        
        
        assert_eq!(tokens[0].token_type, TokenType::Error);
    }


    #[test]
    fn test_number_literals() {
        let source = "123 45.67";
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        assert_eq!(tokens.len(), 3); 
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].lexeme, "123");
        assert_eq!(tokens[1].token_type, TokenType::Number);
        assert_eq!(tokens[1].lexeme, "45.67");
    }

    #[test]
    fn test_keywords_and_identifiers() {
        let source = "spell cast my_spell bind";
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::Spell,
            TokenType::Cast,
            TokenType::Identifier,
            TokenType::Bind,
            TokenType::Eof,
        ];
        
        assert_eq!(tokens.len(), expected_types.len());
        assert_eq!(tokens[0].token_type, TokenType::Spell);
        assert_eq!(tokens[1].token_type, TokenType::Cast);
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].lexeme, "my_spell");
        assert_eq!(tokens[3].token_type, TokenType::Bind);
    }

    #[test]
    fn test_whitespace_and_comments() {
        let source = r#"
           // this whisper shall be ignored!
            mark x = 10; // Another whisper
            // More of a whisper
        "#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::Mark,
            TokenType::Identifier,
            TokenType::Equal,
            TokenType::Number,
            TokenType::SemiColon,
            TokenType::Eof,
        ];
        
        assert_eq!(tokens.len(), expected_types.len());
        for (i, token_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *token_type);
        }
        
        assert_eq!(tokens[0].line, 3); 
    }

    #[test]
    fn test_string_interpolation_identifier() {
        let source = r#""x is @x""#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::String,
            TokenType::InterpolateStart,
            TokenType::Identifier,
            TokenType::InterpolateEnd,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (i, token_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *token_type);
        }

        assert_eq!(tokens[0].lexeme, "x is ");
        assert_eq!(tokens[2].lexeme, "x");
    }

    #[test]
    fn test_string_interpolation_expression() {
        let source = r#""sum @(a + 1)""#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::String,
            TokenType::InterpolateStart,
            TokenType::Identifier,
            TokenType::Plus,
            TokenType::Number,
            TokenType::InterpolateEnd,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (i, token_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *token_type);
        }
    }

    #[test]
    fn test_string_escaped_at() {
        let source = r#""@@home""#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].lexeme, "@home");
    }

    #[test]
    fn test_string_interpolation_nested_parens() {
        let source = r#""@(a + (b))""#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        let expected_types = vec![
            TokenType::InterpolateStart,
            TokenType::Identifier,
            TokenType::Plus,
            TokenType::ParenLeft,
            TokenType::Identifier,
            TokenType::ParenRight,
            TokenType::InterpolateEnd,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (i, token_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *token_type);
        }
    }

    #[test]
    fn test_unterminated_interpolation_expression() {
        let source = r#""@(a + 1""#;
        let scanner = Scanner::init(source);
        let tokens = scanner.tokenize();

        assert_eq!(tokens[0].token_type, TokenType::InterpolateStart);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].token_type, TokenType::Plus);
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[4].token_type, TokenType::Error);
    }
}