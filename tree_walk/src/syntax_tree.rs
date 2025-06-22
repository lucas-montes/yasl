use std::{
    fmt::{Debug, Display},
    rc::Rc,
};
use scan::{Token, TokenLexem, TokenType};
use super::{
    Interpreter,
    interpreter::InterpreterResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOperator {
    Or,
    And,
}

impl From<&TokenType> for LogicalOperator {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Or => Self::Or,
            TokenType::And => Self::And,
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Minus,
    Bang,
}

impl From<&TokenType> for UnaryOperator {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Minus => Self::Minus,
            TokenType::Bang => Self::Bang,
            _ => todo!(),
        }
    }
}

pub trait Callable: Debug {
    fn name(&self) -> TokenLexem;
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: &[Literal]) -> InterpreterResult;
    fn clone_box(&self) -> Box<dyn Callable>;
}

#[derive(Debug)]
pub enum Literal {
    String(Rc<str>),
    Number(f64),
    False,
    True,
    Nil,
    Callable(Box<dyn Callable>),
}

impl Clone for Literal {
    fn clone(&self) -> Self {
        match self {
            Self::String(s) => Self::String(Rc::clone(s)),
            Self::Number(n) => Self::Number(*n),
            Self::False => Self::False,
            Self::True => Self::True,
            Self::Nil => Self::Nil,
            Self::Callable(c) => Self::Callable(c.clone_box()),
        }
    }
}

impl Literal {
    pub fn from_bool(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::False => false,
            Self::True => true,
            Self::Number(v) => !matches!(v, 0.0),
            Self::Nil => false,
            Self::String(v) => !v.is_empty(),
            _ => todo!("add truty for callable or move it out"),
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::False, Self::False) => true,
            (Self::True, Self::True) => true,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::False => write!(f, "False"),
            Self::True => write!(f, "True"),
            Self::Nil => write!(f, "Nil"),
            Self::Callable(c) => write!(f, "fn <{}>", c.name()),
        }
    }
}

impl<'a> From<Token<'a>> for Literal {
    fn from(value: Token<'a>) -> Self {
        match value.kind() {
            TokenType::False => Self::False,
            TokenType::True => Self::True,
            TokenType::Nil => Self::Nil,
            TokenType::String => Self::String(value.value().into()),
            TokenType::Number => Self::Number(value.value().parse().unwrap()),
            _ => todo!(),
        }
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Slash,
    Star,
    Plus,
    Minus,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BangEqual,
    EqualEqual,
}
impl From<&TokenType> for BinaryOperator {
    fn from(value: &TokenType) -> Self {
        match value {
            TokenType::Star => Self::Star,
            TokenType::EqualEqual => Self::EqualEqual,
            TokenType::BangEqual => Self::BangEqual,
            TokenType::Slash => Self::Slash,
            TokenType::Plus => Self::Plus,
            TokenType::Minus => Self::Minus,
            TokenType::Greater => Self::Greater,
            TokenType::GreaterEqual => Self::GreaterEqual,
            TokenType::Less => Self::Less,
            TokenType::LessEqual => Self::LessEqual,
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Assign(TokenLexem, Box<Expr>),
    Literal(Literal),
    Grouping(Box<Expr>),
    Unary(UnaryOperator, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Logical(Box<Expr>, LogicalOperator, Box<Expr>),
    Variable(TokenLexem),
    Call(Box<Expr>, Vec<Expr>),
}

impl Expr {
    pub fn call(callee: Expr, arguments: Vec<Expr>) -> Self {
        Self::Call(Box::new(callee), arguments)
    }

    pub fn logical(expr: Expr, op: LogicalOperator, right: Expr) -> Self {
        Self::Logical(Box::new(expr), op, Box::new(right))
    }

    pub fn binary(expr: Expr, op: BinaryOperator, right: Expr) -> Self {
        Self::Binary(Box::new(expr), op, Box::new(right))
    }

    pub fn unary(op: UnaryOperator, expr: Expr) -> Self {
        Self::Unary(op, Box::new(expr))
    }

    pub fn grouping(expr: Expr) -> Self {
        Self::Grouping(Box::new(expr))
    }

    pub fn assign(token: TokenLexem, expr: Expr) -> Self {
        Self::Assign(token, Box::new(expr))
    }

    pub fn literal(expr: Literal) -> Self {
        Self::Literal(expr)
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Return(Option<Expr>),
    Block(Vec<Stmt>),
    Var(TokenLexem, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Function(TokenLexem, Vec<TokenLexem>, Vec<Stmt>),
}

impl Stmt {
    pub fn while_statement(condition: Expr, body: Stmt) -> Self {
        Self::While(condition, Box::new(body))
    }

    pub fn if_statement(condition: Expr, then: Stmt, else_branch: Option<Stmt>) -> Self {
        Self::If(condition, Box::new(then), else_branch.map(Box::new))
    }
}
