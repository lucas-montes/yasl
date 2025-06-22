pub enum Opcode<T> {
    Constant(T),
    Return, // Return from the current function
Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub struct Chunk<T>{
    code: Vec<Opcode<T>>,
    lines: Vec<usize>,
}
