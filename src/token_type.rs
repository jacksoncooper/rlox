use std::fmt::{self, Display};

use crate::token_type::TokenType as TT;

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

impl Display for TokenType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer: String;

        let readable = match self {
            // Single-character tokens.
            TT::LeftParen => "(",
            TT::RightParen => ")",
            TT::LeftBrace => "{",
            TT::RightBrace => "}",
            TT::Comma => ",",
            TT::Dot => ".",
            TT::Minus => "-",
            TT::Plus => "+",
            TT::Semicolon => ";",
            TT::Slash => "/",
            TT::Star => "*",

            // One- or two-character tokens.
            TT::Bang => "!",
            TT::BangEqual => "!=",
            TT::Equal => "=",
            TT::EqualEqual => "==",
            TT::Greater => ">",
            TT::GreaterEqual => ">=",
            TT::Less => "<",
            TT::LessEqual => "<=",

            // Literals.
            TT::Identifier(text) => {
                buffer = format!("identifier '{}'", text);
                &buffer
            }
            TT::String(text) => {
                buffer = format!("string '{}'", text);
                &buffer
            }
            TT::Number(number) => {
                buffer = format!("number {}", number);
                &buffer
            }

            // Keywords.
            TT::And => "and",
            TT::Class => "class",
            TT::Else => "else",
            TT::False => "false",
            TT::Fun => "fun",
            TT::For => "for",
            TT::If => "if",
            TT::Nil => "nil",
            TT::Or => "or",
            TT::Print => "print",
            TT::Return => "return",
            TT::Super => "super",
            TT::This => "this",
            TT::True => "true",
            TT::Var => "var",
            TT::While => "While",
            TT::EndOfFile => "end of file"
        };

        write!(formatter, "{}", readable)
    }
}
