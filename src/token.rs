use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenKind {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    Comma,
    Dot,
    DotDot,
    Minus,
    Plus,
    Semicolon,
    Colon,
    FSlash,
    Asterisk,
    Incr,
    Decr,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Num,

    And,
    Def,
    Else,
    False,
    For,
    If,
    In,
    Let,
    Not,
    Null,
    Or,
    Print,
    Return,
    Step,
    True,
    While,

    Eof,
    Indent,
    Dedent,
}

#[derive(Debug, PartialEq)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub literal: String,
}

impl TextSpan {
    pub fn new(start: usize, end: usize, literal: String) -> Self {
        return Self {
            start,
            end,
            literal,
        };
    }

    pub fn _length(self) -> usize {
        return self.end - self.start;
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: TextSpan,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, span: TextSpan, line: usize) -> Self {
        return Self {
            kind,
            span,
            line,
        };
    }
}

// pub struct Span {
//     pub start: usize,
//     pub end: usize,
// }

// impl Span {
//     pub fn new(start: usize, end: usize) -> Self {
//         return Self {
//             start,
//             end,
//         };
//     }

//     pub fn _length(self) -> usize {
//         return self.end - self.start;
//     }
// }

impl fmt::Display for TokenKind {
    /// Implements the `Display` trait for `TokenKind` to provide a string representation
    /// of each token type.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::LParen => write!(f, "LParen"),
            TokenKind::RParen => write!(f, "RParen"),
            TokenKind::LBrace => write!(f, "LBrace"),
            TokenKind::RBrace => write!(f, "RBrace"),
            TokenKind::LBrack => write!(f, "LBrack"),
            TokenKind::RBrack => write!(f, "RBrack"),
            TokenKind::Comma => write!(f, "Comma"),
            TokenKind::Dot => write!(f, "Dot"),
            TokenKind::DotDot => write!(f, "DotDot"),
            TokenKind::Minus => write!(f, "Minus"),
            TokenKind::Plus => write!(f, "Plus"),
            TokenKind::Semicolon => write!(f, "Semicolon"),
            TokenKind::Colon => write!(f, "Colon"),
            TokenKind::FSlash => write!(f, "FSlash"),
            TokenKind::Asterisk => write!(f, "Asterisk"),
            TokenKind::Incr => write!(f, "Incr"),
            TokenKind::Decr => write!(f, "Decr"),
            TokenKind::Bang => write!(f, "Bang"),
            TokenKind::BangEqual => write!(f, "BangEqual"),
            TokenKind::Equal => write!(f, "Equal"),
            TokenKind::EqualEqual => write!(f, "EqualEqual"),
            TokenKind::Greater => write!(f, "Greater"),
            TokenKind::GreaterEqual => write!(f, "GreaterEqual"),
            TokenKind::Less => write!(f, "Less"),
            TokenKind::LessEqual => write!(f, "LessEqual"),
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::String => write!(f, "String"),
            TokenKind::Num => write!(f, "Num"),
            TokenKind::And => write!(f, "And"),
            TokenKind::Else => write!(f, "Else"),
            TokenKind::False => write!(f, "False"),
            TokenKind::For => write!(f, "For"),
            TokenKind::Def => write!(f, "Def"),
            TokenKind::If => write!(f, "If"),
            TokenKind::In => write!(f, "In"),
            TokenKind::Let => write!(f, "Let"),
            TokenKind::Not => write!(f, "Not"),
            TokenKind::Null => write!(f, "Null"),
            TokenKind::Or => write!(f, "Or"),
            TokenKind::Print => write!(f, "Print"),
            TokenKind::Return => write!(f, "Return"),
            TokenKind::Step => write!(f, "Step"),
            TokenKind::True => write!(f, "True"),
            TokenKind::While => write!(f, "While"),
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::Indent => write!(f, "Indent"),
            TokenKind::Dedent => write!(f, "Dedent"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "Token{{{}, {}, {}}}", self.kind, self.span, self.line);
    }
}

impl fmt::Display for TextSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "Span{{{}, {}, {}}}", self.start, self.end, self.literal);
    }
}
