use crate::frontend::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

pub struct Scanner<'a> {
    pub source: &'a str,
    start: usize,
    current: usize,
    line: usize,
    column: usize,

    // interpolating: bool,
}

impl<'a> Scanner<'a> {
    pub fn init(source: &'a str) -> Self {
        Self {
            source: source,
            current: 0,
            line: 1,
            start: 0,
            column: 0,
            // interpolating: false,
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current..].chars().next().unwrap();
        self.current += ch.len_utf8();
        self.column += ch.len_utf8();
        return ch;
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        // println!("{:?}", token_type);
        return Token {
            lexeme: self.source[self.start..self.current].to_string(),
            token_type: token_type,
            line: self.line,
            column: self.column,
        };
    }

    fn error_token(&self, msg: &'a str) -> Token {
        return Token {
            lexeme: msg.to_string(),
            line: self.line,
            token_type: TokenType::Error,
            column: self.column,
        };
    }

    fn reached_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn peek(&self) -> Option<char> {
        if self.reached_end() {
            return None;
        }
        return self.source[self.current..].chars().next();
    }

    fn peek_next(&self) -> Option<char> {
        if self.reached_end() {
            return None;
        }
        let mut c = self.source[self.current..].chars();
        c.next();
        return c.next();
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

        return self.make_token(identifier_type(identifier));
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

        return self.make_token(TokenType::Number);
    }

    fn string(&mut self) -> Token {
        while (self.peek() != Some('"')) && !self.reached_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.reached_end() {
            return self.error_token("Dont you think that a string is unterminated?");
        }

        self.advance();

        Token {
            lexeme: self.source[self.start + 1..self.current - 1].to_string(),
            line: self.line,
            column: self.column,
            token_type: TokenType::String,
        }
    }

    fn match_char(&mut self, expect: char) -> bool {
        if self.reached_end() || self.source[self.current..].chars().next().unwrap() != expect {
            return false;
        }
        let ch = self.source[self.current..].chars().next().unwrap();
        self.current += ch.len_utf8();
        return true;
    }

    fn eat_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\r' | '\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.column = 0; //reset the char position tracker
                    self.advance();
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

    fn scan_token(&mut self) -> Token {
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
            ';' => self.make_token(TokenType::SemiColon),

            ':' if self.match_char(':') => self.make_token(TokenType::ColonColon),
            ':' => self.make_token(TokenType::Colon),

            '.' => self.make_token(TokenType::Dot),
            ',' => self.make_token(TokenType::Comma),

            '+' => self.make_token(TokenType::Plus),
            '-' => self.make_token(TokenType::Minus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),

            '!' if self.match_char('=') => self.make_token(TokenType::BangEqual),
            '!' => self.make_token(TokenType::Bang),

            '=' if self.match_char('=') => self.make_token(TokenType::EqualEqual),
            '=' => self.make_token(TokenType::Equal),

            '>' if self.match_char('=') => self.make_token(TokenType::GreaterEqual),
            '>' => self.make_token(TokenType::Greater),

            '<' if self.match_char('=') => self.make_token(TokenType::LessEqual),
            '<' => self.make_token(TokenType::Less),

            '"' => self.string(),
            // '\'' => self.string(),
            _ => self.error_token("error, met an unexpected token."),
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let token = self.scan_token();
            if token.token_type == TokenType::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

fn is_alpha(c: char) -> bool {
    return c.is_ascii_alphabetic() || c == '_';
}

fn is_number(c: char) -> bool {
    return c.is_ascii_digit();
}

fn identifier_type(ident: &str) -> TokenType {
    return match ident {
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
        // "maybe" => TokenType::Maybe,
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
    };
}
