use std::fmt;

use crate::object::Object;
use crate::token::Token;

#[derive(Debug)]

pub enum Expr {
    Assignment { name: Token, value: Box<Expr> },
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Grouping { grouping: Box<Expr> },
    Literal { value: Object },
    Unary { operator: Token, right: Box<Expr> },
    Variable { name: Token },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assignment { name, value } =>
                write!(f, "(= {} {})", name.lexeme, value),
            Expr::Binary { left, operator, right } =>
                write!(f, "({} {} {})", operator.lexeme, left, right),
            Expr::Grouping { grouping } =>
                write!(f, "(group {})", grouping),
            Expr::Literal { value } =>
                write!(f, "{}", value),
            Expr::Unary { operator, right } =>
                write!(f, "({} {})", operator.lexeme, right),
            Expr::Variable { name } =>
                write!(f, "(var {})", name.lexeme),
        }
    }
}
