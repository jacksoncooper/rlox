use std::fmt;

use crate::object::Object;
use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    Assignment(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Object),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Variable(Token),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assignment(name, value) =>
                write!(f, "(= {} {})", name.lexeme, value),
            Expr::Binary(left, operator, right) =>
                write!(f, "({} {} {})", operator.lexeme, left, right),
            Expr::Grouping(grouping) =>
                write!(f, "(group {})", grouping),
            Expr::Literal(value) =>
                match value {
                    Object::String(value) =>
                        write!(f, "\"{}\"", value),
                    _ => write!(f, "{}", value),
                },
            Expr::Logical(left, operator, right) =>
                write!(f, "({} {} {})", operator.lexeme, left, right),
            Expr::Unary(operator, right) =>
                write!(f, "({} {})", operator.lexeme, right),
            Expr::Variable(name) =>
                write!(f, "(var {})", name.lexeme),
        }
    }
}
