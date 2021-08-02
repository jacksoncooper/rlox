use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One- or two-character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier(String), String(String), Number(f64),

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    EndOfFile
}

pub enum Literal {
    Number(f64),
    String(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let readable = match self {
            Literal::String(ref string) => format!("\"{}\"", string),
            Literal::Number(number) => number.to_string(),
        };
        write!(f, "{}", readable)
    }
}

pub fn to_literal(token_type: TokenType) -> Option<Literal> {
    match token_type {
        TokenType::Identifier(name) => Some(Literal::String(name)),
        TokenType::String(string)   => Some(Literal::String(string)),
        TokenType::Number(number)   => Some(Literal::Number(number)),
        _ => None
    }
}
