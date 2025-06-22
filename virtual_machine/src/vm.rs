use super::{chunk::Chunk};

const STACK_MAX: usize = 256;

pub struct Vm<T> {
    //TODO: we'll need either pointers, usize to point to the location or refs
    chunk: Chunk<T>,
    stack: Vec<T> //TODO: use maybeuninit
}

impl<T> Vm<T> {
    pub fn new(chunk: Chunk<T>) -> Self {
        Self {
            chunk,
            stack: Vec::with_capacity(STACK_MAX),
        }
    }
    fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }
    fn push(&mut self, value: T) {
        self.stack.push(value)
    }
    fn run(&mut self){}
    pub fn interpret(&mut self, chunk: Chunk<T>){}
}
