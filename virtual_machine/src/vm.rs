use crate::chunk::Opcode;

use super::chunk::Chunk;

const STACK_MAX: usize = 256;

#[derive(Debug)]
pub enum InterpretError {
    StackUnderflow,
    DivisionByZero,
}

#[derive(Default, Debug)]
pub struct Vm<T> {
    //TODO: we'll need either pointers, usize to point to the location or refs
    chunk: Chunk<T>,
    stack: Vec<T>, //TODO: use maybeuninit
}

impl<T> Vm<T>
where
    T: std::ops::Neg<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Div<Output = T>
        + Default
        + std::cmp::PartialEq,
{
    pub fn interpret(&mut self, chunk: Chunk<T>) -> Result<(), InterpretError> {
        self.chunk = chunk;
        self.run()
    }

    //TODO: handle errors in a list rather than panic
    fn run(&mut self) -> Result<(), InterpretError> {
        while let Some(instruction) = self.chunk.pop() {
            match instruction {
                Opcode::Return => {
                    if self.stack.pop().is_none() {
                        panic!("problem returning from the function, stack is empty");
                    }
                    return Ok(());
                }
                Opcode::Constant(value) => {
                    self.stack.push(value);
                }
                Opcode::Negate => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(-value);
                    } else {
                        panic!("Stack underflow on Negate");
                    }
                }
                Opcode::Add => {
                    self.add();
                }
                Opcode::Subtract => {
                    self.subtract();
                }
                Opcode::Multiply => {
                    self.multiply();
                }
                Opcode::Divide => {
                    self.divide();
                }
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F)
    where
        F: FnOnce(T, T) -> T,
    {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(op(a, b));
        } else {
            panic!("Stack underflow on binary operation");
        }
    }
    fn add(&mut self) {
        self.binary_op(|a, b| a + b);
    }
    fn subtract(&mut self) {
        self.binary_op(|a, b| a - b);
    }
    fn multiply(&mut self) {
        self.binary_op(|a, b| a * b);
    }
    fn divide(&mut self) {
        self.binary_op(|a, b| {
            if b == T::default() {
                panic!("Division by zero");
            }
            a / b
        });
    }
}
