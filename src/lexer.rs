use crate::core::TokenType;

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' && self.peek_next() == Some('/') {
                while let Some(nc) = self.peek() {
                    self.advance();
                    if nc == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<TokenType> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            let c = match self.peek() {
                Some(ch) => ch,
                None => break,
            };

            match c {
                '(' => { self.advance(); tokens.push(TokenType::LeftParen); }
                ')' => { self.advance(); tokens.push(TokenType::RightParen); }
                '{' => { self.advance(); tokens.push(TokenType::LeftBrace); }
                '}' => { self.advance(); tokens.push(TokenType::RightBrace); }
                '[' => { self.advance(); tokens.push(TokenType::LeftBracket); }
                ']' => { self.advance(); tokens.push(TokenType::RightBracket); }
                ';' => { self.advance(); tokens.push(TokenType::Semicolon); }
                ':' => { self.advance(); tokens.push(TokenType::Colon); }
                ',' => { self.advance(); tokens.push(TokenType::Comma); }
                '.' => { self.advance(); tokens.push(TokenType::Dot); }
                '+' => { self.advance(); tokens.push(TokenType::Plus); }
                '*' => { self.advance(); tokens.push(TokenType::Star); }
                '/' => { self.advance(); tokens.push(TokenType::Slash); }
                '%' => { self.advance(); tokens.push(TokenType::Percent); }
                '&' => {
                    self.advance();
                    if self.peek() == Some('&') {
                        self.advance();
                        tokens.push(TokenType::AmpAmp);
                    } else {
                        tokens.push(TokenType::Ampersand);
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') {
                        self.advance();
                        tokens.push(TokenType::PipePipe);
                    } else {
                        panic!("Unexpected single '|'");
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(TokenType::BangEqual);
                    } else {
                        tokens.push(TokenType::Bang);
                    }
                }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(TokenType::EqualEqual);
                    } else {
                        tokens.push(TokenType::Equal);
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(TokenType::LessEqual);
                    } else {
                        tokens.push(TokenType::Less);
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(TokenType::GreaterEqual);
                    } else {
                        tokens.push(TokenType::Greater);
                    }
                }
                '-' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(TokenType::Arrow);
                    } else {
                        tokens.push(TokenType::Minus);
                    }
                }
                '"' => {
                    self.advance();
                    let mut s = String::new();
                    while let Some(sc) = self.peek() {
                        self.advance();
                        if sc == '"' {
                            break;
                        }
                        if sc == '\\' {
                            if let Some(esc) = self.advance() {
                                match esc {
                                    'n' => s.push('\n'),
                                    'r' => s.push('\r'),
                                    't' => s.push('\t'),
                                    '\\' => s.push('\\'),
                                    '"' => s.push('"'),
                                    _ => s.push(esc),
                                }
                            }
                        } else {
                            s.push(sc);
                        }
                    }
                    tokens.push(TokenType::StringLit(s));
                }
                '0'..='9' => {
                    let mut s = String::new();
                    let mut is_float = false;
                    while let Some(nc) = self.peek() {
                        if nc.is_ascii_digit() {
                            s.push(nc);
                            self.advance();
                        } else if nc == '.' && !is_float && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
                            is_float = true;
                            s.push(nc);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if is_float {
                        tokens.push(TokenType::Number(s.parse().unwrap()));
                    } else {
                        tokens.push(TokenType::IntNumber(s.parse().unwrap()));
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut s = String::new();
                    while let Some(nc) = self.peek() {
                        if nc.is_alphanumeric() || nc == '_' {
                            s.push(nc);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    match s.as_str() {
                        "emit" => tokens.push(TokenType::Emit),
                        "when" => tokens.push(TokenType::When),
                        "otherwise" => tokens.push(TokenType::Otherwise),
                        "during" => tokens.push(TokenType::During),
                        "proc" => tokens.push(TokenType::Proc),
                        "yield" => tokens.push(TokenType::Yield),
                        "layout" => tokens.push(TokenType::Layout),
                        "slot" => tokens.push(TokenType::Slot),
                        "claim" => tokens.push(TokenType::Claim),
                        "purge" => tokens.push(TokenType::Purge),
                        "yes" => tokens.push(TokenType::Yes),
                        "no" => tokens.push(TokenType::No),
                        "num" => tokens.push(TokenType::TypeNum),
                        "int" => tokens.push(TokenType::TypeInt),
                        "flag" => tokens.push(TokenType::TypeFlag),
                        "text" => tokens.push(TokenType::TypeText),
                        "none" => tokens.push(TokenType::TypeNone),
                        _ => tokens.push(TokenType::Identifier(s)),
                    }
                }
                other => panic!("Unexpected character: {}", other),
            }
        }

        tokens.push(TokenType::Eof);
        tokens
    }
}