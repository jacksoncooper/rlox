use std::fmt::{self, Display};

use crate::token_type::TokenType;

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: u32
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: u32) -> Token {
        Token { token_type, lexeme, line }
    }
}

impl Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} from lexeme '{}' on line {}",
            self.token_type, self.lexeme, self.line
        )
    }
}
