use std::{ collections::HashMap, iter::Peekable, str::Chars };

use crate::token::{ TextSpan, Token, TokenKind };

use crate::keywords;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    pub tokens: Vec<Token>,
    line: usize,
    pos: usize,
    kw: HashMap<String, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut kw: HashMap<String, TokenKind> = HashMap::new();
        keywords!(
            kw;
            And, Def, Else, False, For, If, In, Let, Not,
            Null, Or, Print, Return, Step, True, While
        );
        return Self {
            input: input.chars().peekable(),
            tokens: vec![],
            line: 0,
            pos: 0,
            kw,
        };
    }

    pub fn tokenise(mut self) -> Vec<Token> {
        let mut curr_pos: usize = 0;
        while let Some(c) = self.input.next() {
            curr_pos = self.pos;
            self.pos += 1;

            match c {
                '(' => self.push_token(TokenKind::LParen, curr_pos),
                ')' => self.push_token(TokenKind::RParen, curr_pos),
                '{' => self.push_token(TokenKind::LBrace, curr_pos),
                '}' => self.push_token(TokenKind::RBrace, curr_pos),
                '[' => self.push_token(TokenKind::LBrack, curr_pos),
                ']' => self.push_token(TokenKind::RBrack, curr_pos),
                ',' => self.push_token(TokenKind::Comma, curr_pos),
                ';' => self.push_token(TokenKind::Semicolon, curr_pos),
                ':' => self.push_token(TokenKind::Colon, curr_pos),
                '*' => self.push_token(TokenKind::Asterisk, curr_pos),
                '.' => {
                    let kind = if self.check_next('.') {
                        TokenKind::DotDot
                    } else {
                        TokenKind::Dot
                    };
                    self.push_token(kind, curr_pos);
                }
                '-' => {
                    let kind = if self.check_next('-') {
                        TokenKind::Decr
                    } else {
                        TokenKind::Minus
                    };
                    self.push_token(kind, curr_pos);
                }
                '+' => {
                    let kind = if self.check_next('+') { TokenKind::Incr } else { TokenKind::Plus };
                    self.push_token(kind, curr_pos);
                }
                '!' => {
                    let kind = if self.check_next('=') {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    };
                    self.push_token(kind, curr_pos);
                }
                '=' => {
                    let kind = if self.check_next('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    self.push_token(kind, curr_pos);
                }
                '<' => {
                    let kind = if self.check_next('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    self.push_token(kind, curr_pos);
                }
                '>' => {
                    let kind = if self.check_next('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    self.push_token(kind, curr_pos);
                }
                '\r' => {
                    while self.check_next('\n') {}
                    if self.input.peek().is_none() {
                        break;
                    }
                    self.line += 1;
                    self.pos = 0;
                }

                '\n' => {
                    self.line += 1;
                    self.pos = 0;
                }
                ' ' | '\t' => {}
                '/' => {
                    if self.check_next('/') {
                        while let Some(c) = self.input.next() {
                            self.pos += 1;
                            if c == '\n' {
                                self.line += 1;
                                self.pos = 0;
                                break;
                            }
                        }
                    } else {
                        self.push_token(TokenKind::FSlash, curr_pos);
                    }
                }

                '"' => {
                    let literal = self.consume_string();
                    self.push_literal_token(TokenKind::String, literal, curr_pos);
                }

                _ => {
                    if self.is_digit(Some(c)) {
                        let n = self.consume_number(c);
                        self.push_literal_token(TokenKind::Num, n.to_string(), curr_pos);
                    } else if self.is_alpha(Some(c)) {
                        let (kind, literal) = self.consume_identifier(c);
                        self.push_literal_token(kind, literal.unwrap_or_default(), curr_pos);
                    } else {
                        println!("UNEXPECTED SYMBOL");
                        return vec![];
                    }
                }
            }
        }

        self.tokens.push(
            Token::new(TokenKind::Eof, TextSpan::new(curr_pos, self.pos, "".to_string()), self.line)
        );

        return self.tokens;
    }

    fn push_token(&mut self, kind: TokenKind, curr_pos: usize) {
        self.tokens.push(
            Token::new(kind, TextSpan::new(curr_pos, self.pos, "".to_string()), self.line)
        );
    }

    fn push_literal_token(&mut self, kind: TokenKind, literal: String, curr_pos: usize) {
        self.tokens.push(Token::new(kind, TextSpan::new(curr_pos, self.pos, literal), self.line));
    }

    fn check_next(&mut self, c: char) -> bool {
        if let Some(&ec) = self.input.peek() {
            if ec == c {
                self.input.next();
                self.pos += 1;
                return true;
            }
        }
        return false;
    }

    fn consume_string(&mut self) -> String {
        let mut string: Vec<char> = vec![];
        let mut terminated = false;
        while self.input.peek().is_some() && !terminated {
            if let Some(c) = self.input.next() {
                self.pos += 1;
                if c == '"' {
                    terminated = true;
                    break;
                }
                if c == '\n' {
                    println!("UNTERMINATED STRING");
                    break;
                }
                string.push(c);
            }
        }

        if !terminated {
            println!("UNTERMINATED STRING");
        }

        return String::from_iter(string);
    }

    fn consume_number(&mut self, first_digit: char) -> f64 {
        let mut num_vec: Vec<char> = vec![first_digit];

        while let Some(&next) = self.input.peek() {
            if next == '.' {
                let mut iter = self.input.clone();
                iter.next();
                if iter.next() == Some('.') {
                    break;
                }
            }

            if next.is_ascii_digit() {
                self.pos += 1;
                num_vec.push(self.input.next().unwrap());
            } else {
                break;
            }
        }

        if let Some(&next) = self.input.peek() {
            if next == '.' {
                let mut iter = self.input.clone();
                iter.next();
                if iter.next() != Some('.') {
                    num_vec.push(self.input.next().unwrap());
                    self.pos += 1;

                    while let Some(&next_digit) = self.input.peek() {
                        if next_digit.is_ascii_digit() {
                            self.pos += 1;
                            num_vec.push(self.input.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        return String::from_iter(num_vec).parse::<f64>().unwrap();
    }

    fn consume_identifier(&mut self, c: char) -> (TokenKind, Option<String>) {
        let mut string: Vec<char> = vec![c];
        while self.input.peek().is_some() && self.is_alpha(None) {
            if let Some(c) = self.input.next() {
                self.pos += 1;
                string.push(c);
            }
        }
        let text = String::from_iter(string);

        match self.kw.get(&text) {
            Some(v) => {
                return (*v, None);
            }
            None => {
                return (TokenKind::Identifier, Some(text));
            }
        };
    }

    fn is_digit(&mut self, n: Option<char>) -> bool {
        if let Some(num) = n {
            return num.is_ascii_digit();
        }
        if let Some(c) = self.input.peek() {
            return c.is_ascii_digit();
        } else {
            return false;
        }
    }

    fn is_alpha(&mut self, c: Option<char>) -> bool {
        if let Some(ch) = c {
            return ch.is_alphanumeric() || ch == '_';
        }
        if let Some(ch) = self.input.peek() {
            return ch.is_alphanumeric() || *ch == '_';
        } else {
            return false;
        }
    }
}

#[macro_export]
macro_rules! keywords {
    ($kw:expr; $($kws:ident),+) => {
        $(
            let key = stringify!($kws).to_lowercase();
            $kw.insert(key, TokenKind::$kws);
        )+
    };
}
