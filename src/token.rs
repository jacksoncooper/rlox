use crate::token_type::TokenType;

#[derive(Debug)]

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Token {
        Token { token_type, lexeme, line }
    }
}
