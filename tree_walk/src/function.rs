use super::{
    Interpreter,
    interpreter::InterpreterResult,
    syntax_tree::{Callable, Literal, Stmt},
    tokens::TokenLexem,
};

#[derive(Debug, Clone)]
pub struct Function {
    name: TokenLexem,
    params: Vec<TokenLexem>,
    body: Vec<Stmt>,
}

impl Function {
    pub fn new(name: TokenLexem, params: Vec<TokenLexem>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
    }
}

impl Callable for Function {
    fn name(&self) -> TokenLexem {
        self.name.clone()
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Literal]) -> InterpreterResult {
        //TODO: not sure if it works
        interpreter.env.push_scope();

        self.params
            .iter()
            .zip(args)
            .for_each(|(p, a)| interpreter.env.define(p.clone(), a.clone()));

        let value = interpreter.evaluate_block(self.body.clone());
        interpreter.env.pop_scope();

        match value? {
            std::ops::ControlFlow::Break(literal) => Ok(literal),
            std::ops::ControlFlow::Continue(literal) => Ok(literal),
        }
    }
}
