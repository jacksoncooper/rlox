use std::fmt;

use crate::scanner::token_type::TokenType as TT;

#[derive(Clone, Debug, PartialEq)]

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

    EndOfFile,
}

impl fmt::Display for TokenType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let readable: String = match self {
            // Single-character tokens.
            TT::LeftParen => "(".to_string(),
            TT::RightParen => ")".to_string(),
            TT::LeftBrace => "{".to_string(),
            TT::RightBrace => "}".to_string(),
            TT::Comma => ",".to_string(),
            TT::Dot => ".".to_string(),
            TT::Minus => "-".to_string(),
            TT::Plus => "+".to_string(),
            TT::Semicolon => ";".to_string(),
            TT::Slash => "/".to_string(),
            TT::Star => "*".to_string(),

            // One- or two-character tokens.
            TT::Bang => "!".to_string(),
            TT::BangEqual => "!=".to_string(),
            TT::Equal => "=".to_string(),
            TT::EqualEqual => "==".to_string(),
            TT::Greater => ">".to_string(),
            TT::GreaterEqual => ">=".to_string(),
            TT::Less => "<".to_string(),
            TT::LessEqual => "<=".to_string(),

            // Literals.
            TT::Identifier(text) => format!("{:?}", text),
            TT::String(text) => format!("{:?}", text),
            TT::Number(number) => format!("{:?}", number),

            // Keywords.
            TT::And => "and".to_string(),
            TT::Class => "class".to_string(),
            TT::Else => "else".to_string(),
            TT::False => "false".to_string(),
            TT::Fun => "fun".to_string(),
            TT::For => "for".to_string(),
            TT::If => "if".to_string(),
            TT::Nil => "nil".to_string(),
            TT::Or => "or".to_string(),
            TT::Print => "print".to_string(),
            TT::Return => "return".to_string(),
            TT::Super => "super".to_string(),
            TT::This => "this".to_string(),
            TT::True => "true".to_string(),
            TT::Var => "var".to_string(),
            TT::While => "While".to_string(),
            TT::EndOfFile => "end of file".to_string(),
        };

        write!(formatter, "{}", readable)
    }
}
