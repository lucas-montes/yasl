use std::{fmt::Display, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    Equal,
    Greater,
    Less,

    BangEqual,
    EqualEqual,
    GreaterEqual,
    LessEqual,

    Identifier,
    Number,
    String,
    False,
    True,
    And,
    Class,
    Else,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    Eof,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct TokenLexem(Rc<str>);

impl Display for TokenLexem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TokenLexem {
    fn from(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    kind: TokenType,
    lexem: &'a str,
    line: u64,
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.lexem == other.lexem && self.line == other.line
    }
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType, lexem: &'a str, line: u64) -> Self {
        Self { kind, lexem, line }
    }
    pub fn eof(line: u64) -> Self {
        Self {
            kind: TokenType::Eof,
            lexem: "",
            line,
        }
    }
    pub fn value(&self) -> &str {
        &self.lexem
    }
    pub fn kind(&self) -> &TokenType {
        &self.kind
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type:{:?} lexem: {} line{}",
            self.kind, self.lexem, self.line
        )
    }
}
