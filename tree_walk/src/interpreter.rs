use std::{
    ops::ControlFlow,
    time::{SystemTime, UNIX_EPOCH},
};

use scan::TokenLexem;

use super::{
    environment::Environment,
    function::Function,
    syntax_tree::{BinaryOperator, Callable, Expr, Literal, LogicalOperator, Stmt, UnaryOperator},
};

#[derive(Debug)]
pub enum InterpreterError {
    UndefinedVariable,
    WrongValue,
    ValueIsNotCallable,
    WrongArgumentsForFunction,
    DivisionByZero,
}

pub type InterpreterResult = Result<Literal, InterpreterError>;

#[derive(Debug)]
struct Clock;

impl Callable for Clock {
    fn name(&self) -> TokenLexem {
        "clock".into()
    }
    fn call(&self, _interpreter: &mut Interpreter, _args: &[Literal]) -> InterpreterResult {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            .into())
    }
    fn arity(&self) -> usize {
        0
    }
    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(Clock {})
    }
}

#[derive(Debug)]
pub struct Interpreter {
    pub(crate) env: Environment,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut env = Environment::default();

        let clock = Literal::Callable(Box::new(Clock {}));
        env.define("clock".into(), clock);
        Self { env }
    }
}

pub type InterpreterFastResult = Result<ControlFlow<Literal, Literal>, InterpreterError>;

impl Interpreter {
    pub fn evaluate(&mut self, stmt: Stmt) -> InterpreterFastResult {
        self.evaluate_statement(stmt)
    }

    fn evaluate_statement(&mut self, stmt: Stmt) -> InterpreterFastResult {
        match stmt {
            Stmt::Return(return_expr) => match return_expr {
                Some(expr) => {
                    let result = self.evaluate_expression(&expr)?;
                    return Ok(ControlFlow::Break(result));
                }
                None => return Ok(ControlFlow::Break(Literal::Nil)),
            },
            Stmt::Expression(expr) => {
                self.evaluate_expression(&expr)?;
            }
            Stmt::Print(expr) => {
                let result = self.evaluate_expression(&expr)?;
                println!("{}", result);
            }
            Stmt::Block(stmts) => return self.evaluate_block(stmts),
            Stmt::While(cond, body) => {
                while self.evaluate_expression(&cond)?.is_truthy() {
                    if let Some(value) = self.evaluate_statement(*body.clone())?.break_value() {
                        return Ok(ControlFlow::Break(value));
                    }
                }
            }
            Stmt::If(cond, then_stmt, else_branch) => {
                if self.evaluate_expression(&cond)?.is_truthy() {
                    return self.evaluate_statement(*then_stmt);
                } else if let Some(else_stmt) = else_branch {
                    return self.evaluate_statement(*else_stmt);
                }
            }
            Stmt::Var(var, expr) => {
                let result = expr
                    .map(|t| self.evaluate_expression(&t))
                    .transpose()?
                    .unwrap_or(Literal::Nil);
                self.env.define(var, result);
            }
            Stmt::Function(name, params, body) => {
                let fun = Function::new(name.clone(), params, body);
                self.env.define(name, Literal::Callable(Box::new(fun)));
            }
        };
        Ok(ControlFlow::Continue(Literal::Nil))
    }

    pub(crate) fn evaluate_block(&mut self, stmts: Vec<Stmt>) -> InterpreterFastResult {
        self.env.push_scope();

        let result = (|| {
            for stmt in stmts {
                match self.evaluate_statement(stmt)? {
                    ControlFlow::Break(value) => return Ok(ControlFlow::Break(value)),
                    ControlFlow::Continue(..) => continue,
                }
            }
            Ok(ControlFlow::Continue(Literal::Nil))
        })();

        self.env.pop_scope();

        result
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> InterpreterResult {
        match expr {
            Expr::Literal(lit) => Ok(lit.to_owned()),
            Expr::Grouping(expr) => self.evaluate_expression(expr),
            Expr::Unary(op, expr) => self.evaluate_unary(op, expr),
            Expr::Binary(exprl, op, exprr) => self.evaluate_binary(op, exprl, exprr),
            Expr::Logical(exprl, op, exprr) => self.evaluate_logical(op, exprl, exprr),
            Expr::Call(callee, args) => self.evaluate_call(callee, args),
            Expr::Assign(token, expr) => {
                let val = self.evaluate_expression(expr)?;
                self.env
                    .assing(token.to_owned(), val)
                    .ok_or(InterpreterError::UndefinedVariable)
            }
            Expr::Variable(token) => self
                .env
                .get(token)
                .map(|t| t.to_owned())
                .ok_or(InterpreterError::UndefinedVariable),
        }
    }

    fn evaluate_call(&mut self, callee: &Expr, arguments: &Vec<Expr>) -> InterpreterResult {
        let callee = self.evaluate_expression(callee)?;

        let arguments: Vec<Literal> = arguments
            .iter()
            .map(|a| self.evaluate_expression(a))
            .collect::<Result<Vec<_>, _>>()?;

        let fun = match callee {
            Literal::Callable(c) => c,
            _ => return Err(InterpreterError::ValueIsNotCallable),
        };

        if arguments.len() != fun.arity() {
            return Err(InterpreterError::WrongArgumentsForFunction);
        }

        fun.call(self, &arguments)
    }

    fn evaluate_logical(
        &mut self,
        op: &LogicalOperator,
        exprl: &Expr,
        exprr: &Expr,
    ) -> InterpreterResult {
        let litl = self.evaluate_expression(exprl)?;
        let is_truthy = litl.is_truthy();
        if op.eq(&LogicalOperator::Or) {
            if is_truthy {
                return Ok(litl);
            }
        } else if !is_truthy {
            return Ok(litl);
        }
        self.evaluate_expression(exprr)
    }

    fn evaluate_binary(
        &mut self,
        op: &BinaryOperator,
        exprl: &Expr,
        exprr: &Expr,
    ) -> InterpreterResult {
        let litl = self.evaluate_expression(exprl)?;
        let litr = self.evaluate_expression(exprr)?;
        match op {
            BinaryOperator::Less => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::from_bool(l < r)),
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::LessEqual => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::from_bool(l <= r)),
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::Greater => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::from_bool(l > r)),
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::GreaterEqual => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::from_bool(l >= r)),
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::EqualEqual => Ok(Literal::from_bool(Self::is_equal(&litl, &litr))),
            BinaryOperator::BangEqual => Ok(Literal::from_bool(!Self::is_equal(&litl, &litr))),
            BinaryOperator::Plus => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::Number(l + r)),
                (Literal::String(l), Literal::String(r)) => {
                    let concatenated = format!("{}{}", l, r);
                    Ok(Literal::String(concatenated.into()))
                }
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::Minus => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::Number(l - r)),
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::Slash => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => {
                    if r == 0.0 {
                        Err(InterpreterError::DivisionByZero)
                    } else {
                        Ok(Literal::Number(l / r))
                    }
                }
                _ => Err(InterpreterError::WrongValue),
            },
            BinaryOperator::Star => match (litl, litr) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::Number(l * r)),
                _ => Err(InterpreterError::WrongValue),
            },
        }
    }

    /// Helper method for equality comparison
    fn is_equal(left: &Literal, right: &Literal) -> bool {
        match (left, right) {
            (Literal::Number(l), Literal::Number(r)) => l == r,
            (Literal::String(l), Literal::String(r)) => l == r,
            (Literal::True, Literal::True)
            | (Literal::False, Literal::False)
            | (Literal::Nil, Literal::Nil) => true,
            _ => false,
        }
    }

    fn evaluate_unary(&mut self, op: &UnaryOperator, expr: &Expr) -> InterpreterResult {
        let lit = self.evaluate_expression(&expr)?;
        match op {
            UnaryOperator::Minus => match lit {
                Literal::Number(v) => Ok(Literal::Number(-v)),
                _ => todo!(),
            },
            //TODO: use the is truthy or leverage this idea
            UnaryOperator::Bang => match lit {
                Literal::False => Ok(Literal::True),
                Literal::True => Ok(Literal::False),
                Literal::Number(v) => {
                    let result = match v {
                        0.0 => Literal::True,
                        _ => Literal::False,
                    };
                    Ok(result)
                }
                Literal::Nil => Ok(Literal::True),
                Literal::String(..) => Ok(Literal::False),
                Literal::Callable(..) => Err(InterpreterError::WrongValue),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter_evaluate() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(5.0))),
            BinaryOperator::Plus,
            Box::new(Expr::Literal(Literal::Number(3.0))),
        );
        let result = interpreter.evaluate_expression(&expr).unwrap();
        assert_eq!(result, Literal::Number(8.0));
    }

    #[test]
    fn test_interpreter_evaluate_division_by_zero() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(5.0))),
            BinaryOperator::Slash,
            Box::new(Expr::Literal(Literal::Number(0.0))),
        );
        let result = interpreter.evaluate_expression(&expr);
        assert!(matches!(result, Err(InterpreterError::DivisionByZero)));
    }
    #[test]
    fn test_interpreter_evaluate_logical_or() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Logical(
            Box::new(Expr::Literal(Literal::False)),
            LogicalOperator::Or,
            Box::new(Expr::Literal(Literal::True)),
        );
        let result = interpreter.evaluate_expression(&expr).unwrap();
        assert_eq!(result, Literal::True);
    }
    #[test]
    fn test_interpreter_evaluate_logical_and() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Logical(
            Box::new(Expr::Literal(Literal::True)),
            LogicalOperator::And,
            Box::new(Expr::Literal(Literal::False)),
        );
        let result = interpreter.evaluate_expression(&expr).unwrap();
        assert_eq!(result, Literal::False);
    }
    #[test]
    fn test_interpreter_evaluate_logical_not() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Unary(UnaryOperator::Bang, Box::new(Expr::Literal(Literal::True)));
        let result = interpreter.evaluate_expression(&expr).unwrap();
        assert_eq!(result, Literal::False);
    }
    #[test]
    fn test_interpreter_evaluate_function_call() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Call(Box::new(Expr::Variable("clock".into())), vec![]);
        let result = interpreter.evaluate_expression(&expr).unwrap();
        assert!(matches!(result, Literal::Number(_)));
    }
    #[test]
    fn test_interpreter_evaluate_function_call_with_args() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Call(
            Box::new(Expr::Variable("clock".into())),
            vec![Expr::Literal(Literal::Number(1.0))],
        );
        let result = interpreter.evaluate_expression(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::WrongArgumentsForFunction)
        ));
    }
    #[test]
    fn test_interpreter_evaluate_function_call_with_wrong_type() {
        let mut interpreter = Interpreter::default();
        let expr = Expr::Call(Box::new(Expr::Literal(Literal::Number(42.0))), vec![]);
        let result = interpreter.evaluate_expression(&expr);
        assert!(matches!(result, Err(InterpreterError::ValueIsNotCallable)));
    }

    #[test]
    fn test_interpreter_evaluate_function_definition_and_call() {
        let mut interpreter = Interpreter::default();
        let function_def = Stmt::Function(
            "myFunction".into(),
            vec!["arg1".into(), "arg2".into()],
            vec![Stmt::Return(Some(Expr::Binary(
                Box::new(Expr::Variable("arg1".into())),
                BinaryOperator::Plus,
                Box::new(Expr::Variable("arg2".into())),
            )))],
        );
        let result = interpreter.evaluate_statement(function_def).unwrap();
        assert!(matches!(result, ControlFlow::Continue(Literal::Nil)));

        let call_expr = Expr::Call(
            Box::new(Expr::Variable("myFunction".into())),
            vec![
                Expr::Literal(Literal::Number(5.0)),
                Expr::Literal(Literal::Number(3.0)),
            ],
        );
        let result = interpreter.evaluate_expression(&call_expr).unwrap();
        assert_eq!(result, Literal::Number(8.0));
    }
}
