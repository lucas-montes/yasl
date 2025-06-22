
pub struct Token{

}

pub struct Parser{
    current: String
}
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,      // + -
    Factor,    // * /
    Unary,     // ! -
    Call,      // . ()
    Primary,
}
