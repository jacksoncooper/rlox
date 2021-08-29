use crate::token_type::TokenType as TT;

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TT,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TT, lexeme: String, line: usize) -> Token {
        Token { token_type, lexeme, line }
    }

    pub fn to_name(&self) -> (&usize, &str) {
        match self.token_type {
            TT::Identifier(ref identifier, ref name) => (identifier, name),
            TT::This(ref identifier) => (identifier, "this"),
            // A panic here represents a failure in the parser.
            _ => panic!("token is not an identifier")
        }
    }
}
