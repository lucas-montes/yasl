use std::ops::Not;

use super::{
    syntax_tree::{Expr, Literal, Stmt},
    tokens::{Token, TokenLexem, TokenType},
};

#[derive(Default, Debug)]
pub struct Parser {
    results: Vec<Stmt>,
    errors: Vec<ParserError>,
}

impl<'a> Parser {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        let mut results = Vec::with_capacity(tokens.len());
        let mut errors = Vec::new();

        for stmt in ParserIter::new(tokens) {
            match stmt {
                Ok(v) => results.push(v),
                Err(err) => errors.push(err),
            }
        }
        Self { results, errors }
    }

    pub fn results(self) -> Vec<Stmt> {
        self.results
    }

    pub fn errors(&self) -> Option<&[ParserError]> {
        self.errors.is_empty().not().then_some(&self.errors)
    }
}

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

type ParserExprResult = Result<Expr, ParserError>;
type ParserResult = Result<Stmt, ParserError>;

struct ParserIter<'a> {
    inner: std::iter::Peekable<std::vec::IntoIter<Token<'a>>>,
}

impl<'a> ParserIter<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            inner: tokens.into_iter().peekable(),
        }
    }

    /// expression -> assignment ;
    fn expression(&mut self) -> ParserExprResult {
        self.assignment()
    }

    /// assignment -> IDENTIFIER "=" assignment | logic_or ;
    fn assignment(&mut self) -> ParserExprResult {
        let expr = self.logic_or()?;
        if self
            .inner
            .next_if(|t| matches!(t.kind(), TokenType::Equal))
            .is_some()
        {
            let value = self.assignment()?;

            match expr {
                Expr::Variable(token) => return Ok(Expr::assign(token, value)),
                _ => return Err(ParserError::MissingAssignment),
            }
        }
        Ok(expr)
    }

    /// primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER;
    fn primary(&mut self) -> ParserExprResult {
        if let Some(token) = self.inner.next_if(|t| {
            matches!(
                t.kind(),
                TokenType::Nil
                    | TokenType::Number
                    | TokenType::String
                    | TokenType::False
                    | TokenType::True
                    | TokenType::LeftParen
                    | TokenType::Identifier
            )
        }) {
            return match token.kind() {
                TokenType::LeftParen => {
                    let expr = self.expression()?;
                    let _ = self
                        .inner
                        .next_if(|t| t.kind().eq(&TokenType::RightParen))
                        .ok_or(ParserError::MissingRightParentesis)?;

                    Ok(Expr::grouping(expr))
                }
                TokenType::Identifier => Ok(Expr::Variable(token.value().into())),
                _ => Ok(Expr::literal(token.into())),
            };
        }
        Err(ParserError::MissingPrimaryValue)
    }

    /// call -> primary ( "(" arguments? ")" )* ;
    fn call(&mut self) -> ParserExprResult {
        let mut expr = self.primary()?;
        loop {
            if self
                .inner
                .next_if(|t| t.kind().eq(&TokenType::LeftParen))
                .is_some()
            {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParserExprResult {
        let mut arguments = Vec::with_capacity(255);

        if self
            .inner
            .peek()
            .is_some_and(|t| t.kind().eq(&TokenType::RightParen).not())
        {
            loop {
                if arguments.len().ge(&255) {
                    return Err(ParserError::TooManyArguments);
                }
                arguments.push(self.expression()?);
                if self
                    .inner
                    .next_if(|t| t.kind().eq(&TokenType::Comma))
                    .is_none()
                {
                    break;
                }
            }
        }

        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::RightParen))
            .ok_or(ParserError::MissingRightParentesis)?;

        Ok(Expr::call(callee, arguments))
    }

    /// unary -> ( "!" | "-" ) unary | call;
    fn unary(&mut self) -> ParserExprResult {
        match self
            .inner
            .next_if(|t| matches!(t.kind(), TokenType::Bang | TokenType::Minus))
        {
            Some(token) => Ok(Expr::unary(token.kind().into(), self.unary()?)),
            None => self.call(),
        }
    }

    /// factor -> unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> ParserExprResult {
        let mut expr = self.unary()?;
        while let Some(token) = self
            .inner
            .next_if(|t| matches!(t.kind(), TokenType::Slash | TokenType::Star))
        {
            expr = Expr::binary(expr, token.kind().into(), self.unary()?)
        }
        Ok(expr)
    }

    /// term -> factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> ParserExprResult {
        let mut expr = self.factor()?;
        while let Some(token) = self
            .inner
            .next_if(|t| matches!(t.kind(), TokenType::Minus | TokenType::Plus))
        {
            expr = Expr::binary(expr, token.kind().into(), self.factor()?)
        }
        Ok(expr)
    }

    /// comparaison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparaison(&mut self) -> ParserExprResult {
        let mut expr = self.term()?;
        while let Some(token) = self.inner.next_if(|t| {
            matches!(
                t.kind(),
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            expr = Expr::binary(expr, token.kind().into(), self.term()?)
        }
        Ok(expr)
    }

    /// equality -> comparaison ( ( "!=" | "==" ) comparaison )* ;
    fn equality(&mut self) -> ParserExprResult {
        let mut expr = self.comparaison()?;
        while let Some(token) = self
            .inner
            .next_if(|t| matches!(t.kind(), TokenType::BangEqual | TokenType::EqualEqual))
        {
            expr = Expr::binary(expr, token.kind().into(), self.comparaison()?)
        }
        Ok(expr)
    }

    fn synchronize(&mut self) {
        for token in self.inner.by_ref() {
            if matches!(
                token.kind(),
                TokenType::Eof
                    | TokenType::Semicolon
                    | TokenType::Class
                    | TokenType::For
                    | TokenType::Fun
                    | TokenType::If
                    | TokenType::Print
                    | TokenType::Return
                    | TokenType::Var
                    | TokenType::While
            ) {
                return;
            }
        }
    }

    /// logic_or -> logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> ParserExprResult {
        let mut expr = self.logic_and()?;

        while let Some(token) = self.inner.next_if(|t| matches!(t.kind(), TokenType::Or)) {
            expr = Expr::logical(expr, token.kind().into(), self.logic_and()?)
        }
        Ok(expr)
    }

    /// logic_and -> equality ( "and" equality )* ;
    fn logic_and(&mut self) -> ParserExprResult {
        let mut expr = self.equality()?;

        while let Some(token) = self.inner.next_if(|t| matches!(t.kind(), TokenType::And)) {
            expr = Expr::logical(expr, token.kind().into(), self.equality()?)
        }
        Ok(expr)
    }

    // Statements

    /// block -> "{" declaration* "}" ;
    fn block(&mut self) -> ParserResult {
        let block = self.get_stmts_in_block()?;
        Ok(Stmt::Block(block))
    }

    fn get_stmts_in_block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let capacity = self.inner.size_hint();
        let mut block = Vec::with_capacity(capacity.1.unwrap_or(capacity.0));

        while self
            .inner
            .peek()
            .is_some_and(|t| !matches!(t.kind(), TokenType::RightBrace | TokenType::Eof))
        {
            block.push(self.declaration()?);
        }

        self.inner
            .next_if(|t| t.kind().eq(&TokenType::RightBrace))
            .ok_or(ParserError::MissingBrace)?;
        Ok(block)
    }

    /// declaration -> funDecl | varDecl | statement;
    fn declaration(&mut self) -> ParserResult {
        if let Some(token) = self
            .inner
            .next_if(|t| matches!(t.kind(), TokenType::Var | TokenType::Fun))
        {
            match token.kind() {
                TokenType::Var => self.variable_declaration(),
                TokenType::Fun => self.function_declaration(),
                _ => unreachable!(),
            }
        } else {
            self.statement()
        }
    }

    /// funDecl -> "fun" function;
    fn function_declaration(&mut self) -> ParserResult {
        self.function()
    }

    /// function -> IDENTIFIER "(" parameters? ")" block ;
    fn function(&mut self) -> ParserResult {
        let token = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Identifier))
            .ok_or(ParserError::MissingIdentifier)?;
        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::LeftParen))
            .ok_or(ParserError::MissingLeftParentesis)?;
        let params = self.parameters()?;

        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::RightParen))
            .ok_or(ParserError::MissingRightParentesis)?;

        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::LeftBrace))
            .ok_or(ParserError::MissingLeftBrace)?;

        let block = self.get_stmts_in_block()?;
        Ok(Stmt::Function(token.value().into(), params, block))
    }

    /// parameters -> IDENTIFIER ( "," IDENTIFIER )* ;
    fn parameters(&mut self) -> Result<Vec<TokenLexem>, ParserError> {
        let mut params = Vec::with_capacity(255);
        if self
            .inner
            .peek()
            .is_some_and(|t| t.kind().eq(&TokenType::RightParen).not())
        {
            loop {
                if params.len().ge(&255) {
                    return Err(ParserError::TooManyArguments);
                }
                params.push(
                    self.inner
                        .next_if(|t| t.kind().eq(&TokenType::Identifier))
                        .ok_or(ParserError::MissingIdentifier)?
                        .value()
                        .into(),
                );
                if self
                    .inner
                    .next_if(|t| t.kind().eq(&TokenType::Comma))
                    .is_none()
                {
                    break;
                }
            }
        }
        Ok(params)
    }

    /// statement -> exprStmt | returnStmt | forStmt | ifStmt | printStmt | whileStmt | block ;
    fn statement(&mut self) -> ParserResult {
        if let Some(token) = self.inner.next_if(|t| {
            matches!(
                t.kind(),
                TokenType::If
                    | TokenType::Print
                    | TokenType::LeftBrace
                    | TokenType::While
                    | TokenType::For
                    | TokenType::Return
            )
        }) {
            match token.kind() {
                TokenType::If => return self.if_statement(),
                TokenType::Print => return self.print_statement(),
                TokenType::LeftBrace => return self.block(),
                TokenType::While => return self.while_statement(),
                TokenType::For => return self.for_statement(),
                TokenType::Return => return self.return_statement(),
                _ => unreachable!("this should not happen"),
            }
        }
        self.expression_statement()
    }

    /// returnStmt -> "return" expression? ";" ;
    fn return_statement(&mut self) -> ParserResult {
        let mut expr = None;
        if self
            .inner
            .peek()
            .is_some_and(|t| t.kind().eq(&TokenType::Semicolon).not())
        {
            expr = Some(self.expression()?);
        }
        self.inner
            .next_if(|t| t.kind().eq(&TokenType::Semicolon))
            .ok_or(ParserError::MissingSemicolon)
            .map(|_| Stmt::Return(expr))
    }

    /// forStmt -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
    fn for_statement(&mut self) -> ParserResult {
        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::LeftParen))
            .ok_or(ParserError::MissingLeftParentesis)?;

        let init = if self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Semicolon))
            .is_some()
        {
            None
        } else if self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Var))
            .is_some()
        {
            Some(self.variable_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = None;
        if self
            .inner
            .peek()
            .is_some_and(|t| t.kind().eq(&TokenType::Semicolon).not())
        {
            condition = Some(self.expression()?);
        }

        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Semicolon))
            .ok_or(ParserError::MissingSemicolon)?;

        let mut increment = None;
        if self
            .inner
            .peek()
            .is_some_and(|t| t.kind().eq(&TokenType::RightParen).not())
        {
            increment = Some(self.expression()?);
        }

        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::RightParen))
            .ok_or(ParserError::MissingRightParentesis)?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)])
        };

        let condi = condition.unwrap_or(Expr::Literal(Literal::True));
        body = Stmt::while_statement(condi, body);

        if let Some(ini) = init {
            body = Stmt::Block(vec![ini, body]);
        }

        Ok(body)
    }

    /// Used for: "(" expression ")"
    fn get_expression_in_parentesis(&mut self) -> ParserExprResult {
        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::LeftParen))
            .ok_or(ParserError::MissingLeftParentesis)?;
        let condition = self.expression()?;
        let _ = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::RightParen))
            .ok_or(ParserError::MissingRightParentesis)?;
        Ok(condition)
    }

    /// whileStmt -> "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> ParserResult {
        let condition = self.get_expression_in_parentesis()?;
        let body = self.statement()?;
        Ok(Stmt::while_statement(condition, body))
    }

    /// ifStmt -> "if" "(" expression ")" statement ("else" statement )? ;
    fn if_statement(&mut self) -> ParserResult {
        let condition = self.get_expression_in_parentesis()?;

        let then = self.statement()?;
        let else_branch = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Else))
            .map(|_| self.statement())
            .transpose()?;

        Ok(Stmt::if_statement(condition, then, else_branch))
    }

    /// varDecl -> var IDENTIFIER ( "=" expression )? ";" ;
    fn variable_declaration(&mut self) -> ParserResult {
        let token = self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Identifier))
            .ok_or(ParserError::MissingIdentifier)?;
        let mut expr = None;
        if self
            .inner
            .next_if(|t| t.kind().eq(&TokenType::Equal))
            .is_some()
        {
            expr = Some(self.expression()?);
        };
        self.consume_semicolon()?;
        Ok(Stmt::Var(token.value().into(), expr))
    }

    fn print_statement(&mut self) -> ParserResult {
        let expr = self.expression()?;
        self.consume_semicolon()?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> ParserResult {
        let expr = self.expression()?;
        self.consume_semicolon()?;
        Ok(Stmt::Expression(expr))
    }

    fn consume_semicolon(&mut self) -> Result<(), ParserError> {
        self.inner
            .next_if(|t| t.kind().eq(&TokenType::Semicolon))
            .ok_or(ParserError::MissingSemicolon)?;
        Ok(())
    }
}

impl<'a> Iterator for ParserIter<'a> {
    type Item = ParserResult;
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.peek() {
            Some(t) => {
                if t.kind().eq(&TokenType::Eof) {
                    self.inner.next()?;
                    return None;
                };
                let declaration = self.declaration();
                //TODO: check if it makes sense
                if declaration.is_err() {
                    self.synchronize();
                }
                Some(declaration)
            }
            None => None,
        }
    }
}
