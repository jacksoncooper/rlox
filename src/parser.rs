use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

impl Parser {
    pub fn new(&self) -> Parser {
        Parser { tokens: Vec::new(), current: 0 }
    }
}
