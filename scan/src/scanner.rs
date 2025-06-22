use std::{fmt::Display, iter::Peekable, ops::Not, str::CharIndices};

use super::tokens::{Token, TokenType};

#[derive(Debug)]
pub enum ScanError {
    UnexpectedCharacter(u64),
    TokenMissing(u64),
}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::UnexpectedCharacter(line) => {
                write!(f, "Unexpected character encountered at line {}", line)
            }
            ScanError::TokenMissing(line) => {
                write!(f, "Token missing at line {}", line)
            }
        }
    }
}

type ScanResult<T> = Result<T, ScanError>;

#[derive(Default, Debug)]
pub struct Scanner<'sourcecode> {
    source: &'sourcecode str,
    tokens: Vec<Token<'sourcecode>>,
    errors: Vec<ScanError>,
}

impl<'sourcecode> Scanner<'sourcecode> {
    pub fn new(source: &'sourcecode str) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }
    pub fn scan(mut self) -> Self {
        for token_result in ScanIter::new(self.source) {
            match token_result {
                Ok(token) => self.tokens.push(token),
                Err(error) => self.errors.push(error),
            }
        }
        self
    }

    pub fn tokens(self) -> Vec<Token<'sourcecode>> {
        self.tokens
    }
    pub fn errors(&self) -> Option<&[ScanError]> {
        self.errors.is_empty().not().then_some(&self.errors)
    }
}

struct ScanIter<'sourcecode> {
    line: u64,
    source: &'sourcecode str,
    inner: Peekable<CharIndices<'sourcecode>>,
    eof_returned: bool,
}

impl<'sourcecode> ScanIter<'sourcecode> {
    pub fn new(source: &'sourcecode str) -> Self {
        Self {
            line: 1,
            source,
            inner: source.char_indices().peekable(),
            eof_returned: false,
        }
    }
}

impl<'sourcecode> Iterator for ScanIter<'sourcecode> {
    type Item = ScanResult<Token<'sourcecode>>;
    fn next(&mut self) -> Option<Self::Item> {
        //TODO: instead of using the source to get the values I could use the encode_utf8 method
        let Some((current_pos, current_char)) = self.inner.next() else {
            if self.eof_returned {
                return None;
            }
            self.eof_returned = true;
            return Some(Ok(Token::eof(self.line)));
        };

        if current_char.is_whitespace() {
            if current_char == '\n' {
                self.line += 1;
            }
            return self.next();
        }

        let next_char = self.inner.peek().map(|n| n.1);
        //TODO: maybe use next_if here too

        match TokenKinds::from_char(current_char, next_char) {
            TokenKinds::SingleChar(token_type) => Some(Ok(Token::new(
                token_type,
                &self.source[current_pos..current_pos + 1],
                self.line,
            ))),
            TokenKinds::DoubleChar(token_type) => {
                let Some((next_pos, _)) = self.inner.next() else {
                    return Some(Err(ScanError::TokenMissing(self.line)));
                };
                Some(Ok(Token::new(
                    token_type,
                    &self.source[current_pos..next_pos + 1],
                    self.line,
                )))
            }
            TokenKinds::Comment => {
                while let Some((_, next_char)) = self.inner.next() {
                    if next_char == '\n' {
                        self.line += 1;
                        break;
                    }
                }
                self.next()
            }
            TokenKinds::NewLine => {
                self.line += 1;
                self.next()
            }
            TokenKinds::String => {
                while let Some((next_pos, next_char)) = self.inner.next() {
                    if next_char == '\n' {
                        self.line += 1;
                    } else if next_char == '"' {
                        //NOTE: we remove the quotes from the string
                        let lexem = &self.source[current_pos + 1..next_pos];
                        return Some(Ok(Token::new(TokenType::String, lexem, self.line)));
                    }
                }
                Some(Err(ScanError::TokenMissing(self.line)))
            }
            TokenKinds::Number => {
                let mut next_pos = current_pos;
                while let Some((next_pos1, next_char)) = self
                    .inner
                    .next_if(|(_, c)| c.is_ascii_digit() || c.eq(&'.'))
                {
                    next_pos = next_pos1;
                    if next_char == '.' {
                        while let Some((next_pos2, _)) =
                            self.inner.next_if(|(_, c)| c.is_ascii_digit())
                        {
                            next_pos = next_pos2;
                        }
                        break;
                    }
                }
                let lexem = &self.source[current_pos..next_pos + 1];
                Some(Ok(Token::new(TokenType::Number, lexem, self.line)))
            }
            TokenKinds::Keyword => {
                let mut next_pos = current_pos;
                while let Some((next_pos1, _)) =
                    self.inner.next_if(|(_, c)| c.is_ascii_alphanumeric())
                {
                    next_pos = next_pos1;
                }
                let lexem = &self.source[current_pos..next_pos + 1];
                let token = match lexem {
                    "and" => TokenType::And,
                    "class" => TokenType::Class,
                    "else" => TokenType::Else,
                    "false" => TokenType::False,
                    "fun" => TokenType::Fun,
                    "for" => TokenType::For,
                    "if" => TokenType::If,
                    "nil" => TokenType::Nil,
                    "or" => TokenType::Or,
                    "print" => TokenType::Print,
                    "return" => TokenType::Return,
                    "super" => TokenType::Super,
                    "this" => TokenType::This,
                    "true" => TokenType::True,
                    "var" => TokenType::Var,
                    "while" => TokenType::While,
                    _ => TokenType::Identifier,
                };
                Some(Ok(Token::new(token, lexem, self.line)))
            }
            TokenKinds::Unknown => Some(Err(ScanError::UnexpectedCharacter(self.line))),
        }
    }
}

enum TokenKinds {
    SingleChar(TokenType),
    DoubleChar(TokenType),
    Comment,
    NewLine,
    String,
    Number,
    Keyword,
    Unknown,
}

impl TokenKinds {
    fn from_char(c: char, next_c: Option<char>) -> Self {
        match (c, next_c) {
            ('(', _) => Self::SingleChar(TokenType::LeftParen),
            (')', _) => Self::SingleChar(TokenType::RightParen),
            ('{', _) => Self::SingleChar(TokenType::LeftBrace),
            ('}', _) => Self::SingleChar(TokenType::RightBrace),
            (',', _) => Self::SingleChar(TokenType::Comma),
            ('.', _) => Self::SingleChar(TokenType::Dot),
            ('-', _) => Self::SingleChar(TokenType::Minus),
            ('+', _) => Self::SingleChar(TokenType::Plus),
            (';', _) => Self::SingleChar(TokenType::Semicolon),
            ('*', _) => Self::SingleChar(TokenType::Star),
            ('!', Some('=')) => Self::DoubleChar(TokenType::BangEqual),
            ('=', Some('=')) => Self::DoubleChar(TokenType::EqualEqual),
            ('>', Some('=')) => Self::DoubleChar(TokenType::GreaterEqual),
            ('<', Some('=')) => Self::DoubleChar(TokenType::LessEqual),
            ('!', _) => Self::SingleChar(TokenType::Bang),
            ('=', _) => Self::SingleChar(TokenType::Equal),
            ('>', _) => Self::SingleChar(TokenType::Greater),
            ('<', _) => Self::SingleChar(TokenType::Less),
            ('/', Some('/')) => Self::Comment,
            ('/', _) => Self::SingleChar(TokenType::Slash),
            ('\n', _) => Self::NewLine,
            ('"', _) => Self::String,
            ('0'..='9', _) => Self::Number,
            ('a'..='z' | 'A'..='Z' | '_', _) => Self::Keyword,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_tokens() {
        let source = r#"1"#;
        let expected_tokens = vec![
            Token::new(TokenType::Number, "1", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_numbers_tokens() {
        let source = r#"123"#;
        let expected_tokens = vec![
            Token::new(TokenType::Number, "123", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_decimals_tokens() {
        let source = r#"1.23"#;
        let expected_tokens = vec![
            Token::new(TokenType::Number, "1.23", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_single_character_tokens() {
        let source = r#"(){}.,-+;*"#;
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::RightBrace, "}", 1),
            Token::new(TokenType::Dot, ".", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Minus, "-", 1),
            Token::new(TokenType::Plus, "+", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::Star, "*", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_single_character_tokens_multiline() {
        let source = "(){}.,\n-+;*";
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::RightBrace, "}", 1),
            Token::new(TokenType::Dot, ".", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Minus, "-", 2),
            Token::new(TokenType::Plus, "+", 2),
            Token::new(TokenType::Semicolon, ";", 2),
            Token::new(TokenType::Star, "*", 2),
            Token::new(TokenType::Eof, "", 2),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_simple_characters_tokens_with_comment() {
        let source = r#"()//!=.==>=*"#;
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_simple_characters_tokens_with_comment_and_new_line() {
        let source = "()//!=.==>\n=*";
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::Equal, "=", 2),
            Token::new(TokenType::Star, "*", 2),
            Token::new(TokenType::Eof, "", 2),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_double_character_tokens() {
        let source = r#"()!=.==>=/"#;
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::BangEqual, "!=", 1),
            Token::new(TokenType::Dot, ".", 1),
            Token::new(TokenType::EqualEqual, "==", 1),
            Token::new(TokenType::GreaterEqual, ">=", 1),
            Token::new(TokenType::Slash, "/", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_string_tokens() {
        let source = "()\"hey, como\"";
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::String, "hey, como", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_string_tokens_new_line() {
        let source = "()\"hey,\n como\"";
        let expected_tokens = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::String, "hey,\n como", 2),
            Token::new(TokenType::Eof, "", 2),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_simple_function_declaration() {
        let source = r#"fun greet(name) {
        print "Hello, " + name + "!";
        return nil;
    }"#;
        let expected_tokens = vec![
            Token::new(TokenType::Fun, "fun", 1),
            Token::new(TokenType::Identifier, "greet", 1),
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::Identifier, "name", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::Print, "print", 2),
            Token::new(TokenType::String, "Hello, ", 2),
            Token::new(TokenType::Plus, "+", 2),
            Token::new(TokenType::Identifier, "name", 2),
            Token::new(TokenType::Plus, "+", 2),
            Token::new(TokenType::String, "!", 2),
            Token::new(TokenType::Semicolon, ";", 2),
            Token::new(TokenType::Return, "return", 3),
            Token::new(TokenType::Nil, "nil", 3),
            Token::new(TokenType::Semicolon, ";", 3),
            Token::new(TokenType::RightBrace, "}", 4),
            Token::new(TokenType::Eof, "", 4),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_class_with_methods() {
        let source = r#"class Person {
    init(name, age) {
        this.name = name;
        this.age = age;
    }

    sayHello() {
        print "Hi, I'm " + this.name;
    }
}"#;
        let expected_tokens = vec![
            Token::new(TokenType::Class, "class", 1),
            Token::new(TokenType::Identifier, "Person", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::Identifier, "init", 2),
            Token::new(TokenType::LeftParen, "(", 2),
            Token::new(TokenType::Identifier, "name", 2),
            Token::new(TokenType::Comma, ",", 2),
            Token::new(TokenType::Identifier, "age", 2),
            Token::new(TokenType::RightParen, ")", 2),
            Token::new(TokenType::LeftBrace, "{", 2),
            Token::new(TokenType::This, "this", 3),
            Token::new(TokenType::Dot, ".", 3),
            Token::new(TokenType::Identifier, "name", 3),
            Token::new(TokenType::Equal, "=", 3),
            Token::new(TokenType::Identifier, "name", 3),
            Token::new(TokenType::Semicolon, ";", 3),
            Token::new(TokenType::This, "this", 4),
            Token::new(TokenType::Dot, ".", 4),
            Token::new(TokenType::Identifier, "age", 4),
            Token::new(TokenType::Equal, "=", 4),
            Token::new(TokenType::Identifier, "age", 4),
            Token::new(TokenType::Semicolon, ";", 4),
            Token::new(TokenType::RightBrace, "}", 5),
            Token::new(TokenType::Identifier, "sayHello", 7),
            Token::new(TokenType::LeftParen, "(", 7),
            Token::new(TokenType::RightParen, ")", 7),
            Token::new(TokenType::LeftBrace, "{", 7),
            Token::new(TokenType::Print, "print", 8),
            Token::new(TokenType::String, "Hi, I'm ", 8),
            Token::new(TokenType::Plus, "+", 8),
            Token::new(TokenType::This, "this", 8),
            Token::new(TokenType::Dot, ".", 8),
            Token::new(TokenType::Identifier, "name", 8),
            Token::new(TokenType::Semicolon, ";", 8),
            Token::new(TokenType::RightBrace, "}", 9),
            Token::new(TokenType::RightBrace, "}", 10),
            Token::new(TokenType::Eof, "", 10),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_for_loop_with_counter() {
        let source = r#"for (var i = 0; i < 10; i = i + 1) {
    print i;
}"#;
        let expected_tokens = vec![
            Token::new(TokenType::For, "for", 1),
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::Var, "var", 1),
            Token::new(TokenType::Identifier, "i", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::Number, "0", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::Identifier, "i", 1),
            Token::new(TokenType::Less, "<", 1),
            Token::new(TokenType::Number, "10", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::Identifier, "i", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::Identifier, "i", 1),
            Token::new(TokenType::Plus, "+", 1),
            Token::new(TokenType::Number, "1", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::Print, "print", 2),
            Token::new(TokenType::Identifier, "i", 2),
            Token::new(TokenType::Semicolon, ";", 2),
            Token::new(TokenType::RightBrace, "}", 3),
            Token::new(TokenType::Eof, "", 3),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_fibonacci_function() {
        let source = r#"fun fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}"#;
        let expected_tokens = vec![
            Token::new(TokenType::Fun, "fun", 1),
            Token::new(TokenType::Identifier, "fibonacci", 1),
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::Identifier, "n", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::If, "if", 2),
            Token::new(TokenType::LeftParen, "(", 2),
            Token::new(TokenType::Identifier, "n", 2),
            Token::new(TokenType::LessEqual, "<=", 2),
            Token::new(TokenType::Number, "1", 2),
            Token::new(TokenType::RightParen, ")", 2),
            Token::new(TokenType::Return, "return", 2),
            Token::new(TokenType::Identifier, "n", 2),
            Token::new(TokenType::Semicolon, ";", 2),
            Token::new(TokenType::Return, "return", 3),
            Token::new(TokenType::Identifier, "fibonacci", 3),
            Token::new(TokenType::LeftParen, "(", 3),
            Token::new(TokenType::Identifier, "n", 3),
            Token::new(TokenType::Minus, "-", 3),
            Token::new(TokenType::Number, "1", 3),
            Token::new(TokenType::RightParen, ")", 3),
            Token::new(TokenType::Plus, "+", 3),
            Token::new(TokenType::Identifier, "fibonacci", 3),
            Token::new(TokenType::LeftParen, "(", 3),
            Token::new(TokenType::Identifier, "n", 3),
            Token::new(TokenType::Minus, "-", 3),
            Token::new(TokenType::Number, "2", 3),
            Token::new(TokenType::RightParen, ")", 3),
            Token::new(TokenType::Semicolon, ";", 3),
            Token::new(TokenType::RightBrace, "}", 4),
            Token::new(TokenType::Eof, "", 4),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_variable_assignments_and_comparisons() {
        let source = r#"var x = 42;
var y = 3.14;
var isEqual = x == y;
var isGreater = x > y;
var name = "Alice";
var isValid = name != nil and x >= 0;"#;
        let expected_tokens = vec![
            Token::new(TokenType::Var, "var", 1),
            Token::new(TokenType::Identifier, "x", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::Number, "42", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::Var, "var", 2),
            Token::new(TokenType::Identifier, "y", 2),
            Token::new(TokenType::Equal, "=", 2),
            Token::new(TokenType::Number, "3.14", 2),
            Token::new(TokenType::Semicolon, ";", 2),
            Token::new(TokenType::Var, "var", 3),
            Token::new(TokenType::Identifier, "isEqual", 3),
            Token::new(TokenType::Equal, "=", 3),
            Token::new(TokenType::Identifier, "x", 3),
            Token::new(TokenType::EqualEqual, "==", 3),
            Token::new(TokenType::Identifier, "y", 3),
            Token::new(TokenType::Semicolon, ";", 3),
            Token::new(TokenType::Var, "var", 4),
            Token::new(TokenType::Identifier, "isGreater", 4),
            Token::new(TokenType::Equal, "=", 4),
            Token::new(TokenType::Identifier, "x", 4),
            Token::new(TokenType::Greater, ">", 4),
            Token::new(TokenType::Identifier, "y", 4),
            Token::new(TokenType::Semicolon, ";", 4),
            Token::new(TokenType::Var, "var", 5),
            Token::new(TokenType::Identifier, "name", 5),
            Token::new(TokenType::Equal, "=", 5),
            Token::new(TokenType::String, "Alice", 5),
            Token::new(TokenType::Semicolon, ";", 5),
            Token::new(TokenType::Var, "var", 6),
            Token::new(TokenType::Identifier, "isValid", 6),
            Token::new(TokenType::Equal, "=", 6),
            Token::new(TokenType::Identifier, "name", 6),
            Token::new(TokenType::BangEqual, "!=", 6),
            Token::new(TokenType::Nil, "nil", 6),
            Token::new(TokenType::And, "and", 6),
            Token::new(TokenType::Identifier, "x", 6),
            Token::new(TokenType::GreaterEqual, ">=", 6),
            Token::new(TokenType::Number, "0", 6),
            Token::new(TokenType::Semicolon, ";", 6),
            Token::new(TokenType::Eof, "", 6),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }

    #[test]
    fn test_while_loop_with_conditions() {
        let source = r#"var count = 0;
while (count < 5 and count >= 0) {
    print "Count: " + count;
    count = count + 1;
    if (count == 3) {
        print "Reached middle!";
    }
}"#;
        let expected_tokens = vec![
            Token::new(TokenType::Var, "var", 1),
            Token::new(TokenType::Identifier, "count", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::Number, "0", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::While, "while", 2),
            Token::new(TokenType::LeftParen, "(", 2),
            Token::new(TokenType::Identifier, "count", 2),
            Token::new(TokenType::Less, "<", 2),
            Token::new(TokenType::Number, "5", 2),
            Token::new(TokenType::And, "and", 2),
            Token::new(TokenType::Identifier, "count", 2),
            Token::new(TokenType::GreaterEqual, ">=", 2),
            Token::new(TokenType::Number, "0", 2),
            Token::new(TokenType::RightParen, ")", 2),
            Token::new(TokenType::LeftBrace, "{", 2),
            Token::new(TokenType::Print, "print", 3),
            Token::new(TokenType::String, "Count: ", 3),
            Token::new(TokenType::Plus, "+", 3),
            Token::new(TokenType::Identifier, "count", 3),
            Token::new(TokenType::Semicolon, ";", 3),
            Token::new(TokenType::Identifier, "count", 4),
            Token::new(TokenType::Equal, "=", 4),
            Token::new(TokenType::Identifier, "count", 4),
            Token::new(TokenType::Plus, "+", 4),
            Token::new(TokenType::Number, "1", 4),
            Token::new(TokenType::Semicolon, ";", 4),
            Token::new(TokenType::If, "if", 5),
            Token::new(TokenType::LeftParen, "(", 5),
            Token::new(TokenType::Identifier, "count", 5),
            Token::new(TokenType::EqualEqual, "==", 5),
            Token::new(TokenType::Number, "3", 5),
            Token::new(TokenType::RightParen, ")", 5),
            Token::new(TokenType::LeftBrace, "{", 5),
            Token::new(TokenType::Print, "print", 6),
            Token::new(TokenType::String, "Reached middle!", 6),
            Token::new(TokenType::Semicolon, ";", 6),
            Token::new(TokenType::RightBrace, "}", 7),
            Token::new(TokenType::RightBrace, "}", 8),
            Token::new(TokenType::Eof, "", 8),
        ];
        let scanner = Scanner::new(source);
        assert_eq!(scanner.scan().tokens, expected_tokens);
    }
}
