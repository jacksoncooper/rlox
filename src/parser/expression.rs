use std::fmt;

use crate::interpreter::object::Object;
use crate::scanner::token::Token;

#[derive(Debug)]

pub enum Expr {
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Grouping { grouping: Box<Expr> },
    Literal { value: Object },
    Unary { operator: Token, right: Box<Expr> },
    Variable { name: Token },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary { left, operator, right } =>
                write!(f, "({} {} {})", operator.lexeme, left.to_string(), right.to_string()),
            Expr::Grouping { grouping } =>
                write!(f, "(group {})", grouping.to_string()),
            Expr::Literal { value } =>
                write!(f, "{}", value.to_string()),
            Expr::Unary { operator, right } =>
                write!(f, "({} {})", operator.lexeme, right.to_string()),
            Expr::Variable { name } =>
                write!(f, "(var {})", name.lexeme),
        }
    }
}
