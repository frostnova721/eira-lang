use std::{collections::VecDeque, fmt::{Display}};

use crate::compiler::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn dummy() -> Self {
        Token {
            token_type: TokenType::Error,
            lexeme: "".to_string(),
            line: 0,
            column: 0,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}('{}') at {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ScanMode {
    Normal,
    InString { quote: char },
    InInterpolation { quote: char, paren_depth: usize },
}

pub struct Scanner<'a> {
    pub source: &'a str,
    start: usize,
    current: usize,
    line: usize,
    column: usize,

    mode: ScanMode,
    token_buffer: VecDeque<Token>,
}

impl<'a> Scanner<'a> {
    pub fn init(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            line: 1,
            start: 0,
            column: 0,
            mode: ScanMode::Normal,
            token_buffer: VecDeque::new(),
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current..].chars().next().unwrap();
        self.current += ch.len_utf8();
        self.column += 1;
        ch
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        Token {
            lexeme: self.source[self.start..self.current].to_string(),
            token_type,
            line: self.line,
            column: self.column,
        }
    }

    fn make_token_with_lexeme(&self, token_type: TokenType, lexeme: String) -> Token {
        Token {
            lexeme,
            token_type,
            line: self.line,
            column: self.column,
        }
    }

    fn error_token(&self, msg: &'a str) -> Token {
        Token {
            lexeme: msg.to_string(),
            line: self.line,
            token_type: TokenType::Error,
            column: self.column,
        }
    }

    fn reached_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        if self.reached_end() {
            return None;
        }
        self.source[self.current..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        if self.reached_end() {
            return None;
        }
        let mut c = self.source[self.current..].chars();
        c.next();
        c.next()
    }

    fn identifier(&mut self) -> Token {
        while let Some(c) = self.peek() {
            if is_alpha(c) || is_number(c) {
                self.advance();
            } else {
                break;
            }
        }

        let identifier = &self.source[self.start..self.current];
        self.make_token(identifier_type(identifier))
    }

    fn number(&mut self) -> Token {
        while let Some(c) = self.peek() {
            if is_number(c) {
                self.advance();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') && self.peek_next().map_or(false, is_number) {
            self.advance();

            while let Some(c) = self.peek() {
                if is_number(c) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.make_token(TokenType::Number)
    }

    fn scan_string_mode(&mut self, quote: char) -> Token {
        let mut str_chunk = String::new();

        while !self.reached_end() {
            if self.peek() == Some(quote) {
                self.advance();

                // if !str_chunk.is_empty() {
                    self.token_buffer.push_back(
                        self.make_token_with_lexeme(TokenType::String, str_chunk.clone()),
                    );
                // }

                self.mode = ScanMode::Normal;

                if let Some(token) = self.token_buffer.pop_front() {
                    return token;
                }

                return self.scan_token();
            }

            if self.peek() == Some('@') {
                self.advance();

                if self.peek() == Some('@') {
                    self.advance();
                    str_chunk.push('@');
                    continue;
                }

                if self.peek() == Some('(') {
                    // if the interpolation is starting, push the current string to token buffer
                    if !str_chunk.is_empty() {
                        self.token_buffer.push_back(
                            self.make_token_with_lexeme(TokenType::String, str_chunk.clone()),
                        );
                        str_chunk.clear();
                    }

                    self.advance();
                    self.token_buffer.push_back(
                        self.make_token_with_lexeme(TokenType::InterpolateStart, "@(".to_owned()),
                    );
                    self.mode = ScanMode::InInterpolation {
                        quote,
                        paren_depth: 1,
                    };

                    return self.token_buffer.pop_front().unwrap();
                }

                if let Some(c) = self.peek() {
                    if is_alpha(c) {
                        if !str_chunk.is_empty() {
                            self.token_buffer.push_back(
                                self.make_token_with_lexeme(TokenType::String, str_chunk.clone()),
                            );
                            str_chunk.clear();
                        }

                        let ident_start = self.current;
                        while let Some(ch) = self.peek() {
                            if is_alpha(ch) || is_number(ch) {
                                self.advance();
                            } else {
                                break;
                            }
                        }

                        let ident = self.source[ident_start..self.current].to_string();
                        
                        // let ident_type = identifier_type(&ident);

                        self.token_buffer.push_back(
                            self.make_token_with_lexeme(TokenType::InterpolateStart, "@".to_owned()),
                        );
                        self.token_buffer.push_back(self.make_token_with_lexeme(TokenType::Identifier, ident.clone()));
                        self.token_buffer.push_back(
                            self.make_token_with_lexeme(TokenType::InterpolateEnd, "".to_owned()),
                        );

                        return self.token_buffer.pop_front().unwrap();
                    }
                }

                str_chunk.push('@');
                continue;
            }

            let character = self.advance();
            if character == '\n' {
                self.line += 1;
                self.column = 0;
            }
            str_chunk.push(character);
        }

        self.mode = ScanMode::Normal;
        self.error_token("Don't you think that a string is unterminated?")
    }

    fn scan_interpolation_mode(&mut self, quote: char, mut paren_depth: usize) -> Token {
        self.eat_whitespace();
        self.start = self.current;

        if self.reached_end() {
            self.mode = ScanMode::Normal;
            return self.error_token("Interpolation expression wasn't closed.");
        }

        let c = self.advance();

        if is_alpha(c) {
            return self.identifier();
        }
        if is_number(c) {
            return self.number();
        }

        match c {
            '(' => {
                paren_depth += 1;
                self.mode = ScanMode::InInterpolation { quote, paren_depth };
                self.make_token(TokenType::ParenLeft)
            }
            ')' => {
                if paren_depth == 1 {
                    self.mode = ScanMode::InString { quote };
                    self.make_token_with_lexeme(TokenType::InterpolateEnd, ")".to_owned())
                } else {
                    paren_depth -= 1;
                    self.mode = ScanMode::InInterpolation { quote, paren_depth };
                    self.make_token(TokenType::ParenRight)
                }
            }
            '{' => self.make_token(TokenType::BraceLeft),
            '}' => self.make_token(TokenType::BraceRight),
            '[' => self.make_token(TokenType::SquareLeft),
            ']' => self.make_token(TokenType::SquareRight),
            ';' => self.make_token(TokenType::SemiColon),

            ':' if self.match_char(':') => self.make_token(TokenType::ColonColon),
            ':' => self.make_token(TokenType::Colon),

            '.' => self.make_token(TokenType::Dot),
            ',' => self.make_token(TokenType::Comma),

            '+' => self.make_token(TokenType::Plus),
            '-' => self.make_token(TokenType::Minus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '%' => self.make_token(TokenType::Percent),

            '!' if self.match_char('=') => self.make_token(TokenType::BangEqual),
            '!' => self.make_token(TokenType::Bang),

            '=' if self.match_char('=') => self.make_token(TokenType::EqualEqual),
            '=' => self.make_token(TokenType::Equal),

            '>' if self.match_char('=') => self.make_token(TokenType::GreaterEqual),
            '>' => self.make_token(TokenType::Greater),

            '<' if self.match_char('=') => self.make_token(TokenType::LessEqual),
            '<' => self.make_token(TokenType::Less),

            '~' => self.make_token(TokenType::Tilde),
            '"' => {
                self.error_token("Direct strings on interpolation is'nt supported.")
                // self.mode = ScanMode::InString { quote: '"' };
                // self.scan_string_mode('"')
            }
            '\'' => {
                self.error_token("Direct strings on interpolation is'nt supported.")
                // self.mode = ScanMode::InString { quote: '\'' };
                // self.scan_string_mode('\'')
            }
            _ => self.error_token("met an unexpected token."),
        }
    }

    fn match_char(&mut self, expect: char) -> bool {
        if self.reached_end() || self.source[self.current..].chars().next().unwrap() != expect {
            return false;
        }
        let ch = self.source[self.current..].chars().next().unwrap();
        self.current += ch.len_utf8();
        self.column += 1;
        true
    }

    fn eat_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\r' | '\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                    self.line += 1;
                    self.column = 0;
                }
                Some('/') => {
                    if self.peek_next() == Some('/') {
                        while self.peek() != Some('\n') && !self.reached_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_normal_token(&mut self) -> Token {
        self.eat_whitespace();
        self.start = self.current;
        if self.reached_end() {
            return self.make_token(TokenType::Eof);
        }
        let c = self.advance();

        if is_alpha(c) {
            return self.identifier();
        }
        if is_number(c) {
            return self.number();
        }

        match c {
            '(' => self.make_token(TokenType::ParenLeft),
            ')' => self.make_token(TokenType::ParenRight),
            '{' => self.make_token(TokenType::BraceLeft),
            '}' => self.make_token(TokenType::BraceRight),
            '[' => self.make_token(TokenType::SquareLeft),
            ']' => self.make_token(TokenType::SquareRight),
            ';' => self.make_token(TokenType::SemiColon),

            ':' if self.match_char(':') => self.make_token(TokenType::ColonColon),
            ':' => self.make_token(TokenType::Colon),

            '.' => self.make_token(TokenType::Dot),
            ',' => self.make_token(TokenType::Comma),

            '+' => self.make_token(TokenType::Plus),
            '-' => self.make_token(TokenType::Minus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '%' => self.make_token(TokenType::Percent),

            '!' if self.match_char('=') => self.make_token(TokenType::BangEqual),
            '!' => self.make_token(TokenType::Bang),

            '=' if self.match_char('=') => self.make_token(TokenType::EqualEqual),
            '=' => self.make_token(TokenType::Equal),

            '>' if self.match_char('=') => self.make_token(TokenType::GreaterEqual),
            '>' => self.make_token(TokenType::Greater),

            '<' if self.match_char('=') => self.make_token(TokenType::LessEqual),
            '<' => self.make_token(TokenType::Less),

            '~' => self.make_token(TokenType::Tilde),
            '"' => {
                self.mode = ScanMode::InString { quote: '"' };
                self.scan_string_mode('"')
            }
            '\'' => {
                self.mode = ScanMode::InString { quote: '\'' };
                self.scan_string_mode('\'')
            }
            _ => self.error_token("error, met an unexpected token."),
        }
    }

    fn scan_token(&mut self) -> Token {
        if let Some(token) = self.token_buffer.pop_front() {
            return token;
        }

        match self.mode {
            ScanMode::Normal => self.scan_normal_token(),
            ScanMode::InString { quote } => self.scan_string_mode(quote),
            ScanMode::InInterpolation { quote, paren_depth } => {
                self.scan_interpolation_mode(quote, paren_depth)
            }
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let previous_current = self.current;
            let previous_mode = self.mode;
            
            let token = self.scan_token();
            if token.token_type == TokenType::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);

            if tokens.last().is_some_and(|t| t.token_type == TokenType::Error)
                && self.current == previous_current
                && self.mode == previous_mode
            {
                self.mode = ScanMode::Normal;
                self.start = self.current;
                tokens.push(self.make_token(TokenType::Eof));
                break;
            }
        }
        tokens
    }
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_number(c: char) -> bool {
    c.is_ascii_digit()
}

fn identifier_type(ident: &str) -> TokenType {
    match ident {
        "attune" => TokenType::Attune,
        "bind" => TokenType::Bind,
        "cast" => TokenType::Cast,
        "channel" => TokenType::Channel,
        "chant" => TokenType::Chant,
        "divert" => TokenType::Divert,
        "false" => TokenType::False,
        "fate" => TokenType::Fate,
        "flow" => TokenType::Flow,
        "forge" => TokenType::Forge,
        "mark" => TokenType::Mark,
        "origin" => TokenType::Origin,
        "refers" => TokenType::Refers,
        "release" => TokenType::Release,
        "seal" => TokenType::Seal,
        "self" => TokenType::_Self,
        "secret" => TokenType::Secret,
        "sign" => TokenType::Sign,
        "spell" => TokenType::Spell,
        "sever" => TokenType::Sever,
        "tome" => TokenType::Tome,
        "true" => TokenType::True,
        "while" => TokenType::While,
        "with" => TokenType::With,
        _ => TokenType::Identifier,
    }
}
