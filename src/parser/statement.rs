use std::fmt;

use crate::parser::expression::Expr;

#[derive(Debug)]

pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression { expression } =>
                write!(f, "(expr {})", expression.to_string()),
            Stmt::Print { expression } =>
                write!(f, "(print {})", expression.to_string()),
        }
    }
}
