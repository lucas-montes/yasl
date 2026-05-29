use std::marker::PhantomData;

use super::{
    chunk::{Chunk, Opcode},
    scanner::Scanner,
    tokens::Token,
};

pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

type ParserResult<T> = Result<Opcode<T>, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    MissingLeftBrace,
    MissingLeftParentesis,
    TooManyArguments,
    MissingBrace,
    MissingIdentifier,
    MissingSemicolon,
    MissingAssignment,
    MissingPrimaryValue,
    MissingRightParentesis,
}

pub struct Parser<'a, T> {
    inner: std::iter::Peekable<std::vec::IntoIter<Token<'a>>>,
    had_error: bool,
    panic_mode: bool,
    _phantom: PhantomData<T>,
}

impl<'a, T> Parser<'a, T> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            inner: tokens.into_iter().peekable(),
            had_error: false,
            panic_mode: false,
            _phantom: PhantomData,
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {}
}

impl<'a, T> Iterator for Parser<'a, T> {
    type Item = ParserResult<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let previous = self.inner.next();
        let current = self.inner.next();
        None
    }
}

pub fn compile<T>(scanner: Scanner, chunk: &mut Chunk<T>) {
    let tokens = scanner.tokens();
    let parser: Parser<'_, T> = Parser::new(tokens);
    for op in parser {}
}
