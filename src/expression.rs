use std::fmt;

use crate::object::Object;
use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    Assignment(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
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
            Expr::Call(callee, _, expressions) => {
                let arguments: String = expressions
                    .into_iter()
                    .map(|e| e.to_string())
                    .fold(String::new(), |a, s| format!("{} {}", a, s));
                write!(f, "(call {}{})", callee, arguments)
            },
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
