#[derive(Debug)]
pub enum Opcode<T> {
    Constant(T),
    Return, // Return from the current function
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Default, Debug)]
pub struct Chunk<T> {
    code: Vec<Opcode<T>>,
    lines: Vec<usize>,
}

impl<T> Chunk<T> {
    pub fn push(&mut self, code: Opcode<T>, line: usize) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn pop(&mut self) -> Option<Opcode<T>> {
        self.code.pop()
    }
}
