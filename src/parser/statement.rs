use std::fmt;

use crate::parser::expression::Expr;
use crate::scanner::token::Token;

#[derive(Debug)]

pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Option<Expr> },
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression { expression } =>
                write!(f, "(expr {})", expression.to_string()),
            Stmt::Print { expression } =>
                write!(f, "(print {})", expression.to_string()),
            Stmt::Var { name, initializer } =>
                match initializer {
                    Some(initializer) =>
                        write!(f, "(var {} {})", name.lexeme, initializer.to_string()),
                    None =>
                        write!(f, "(var {})", name.lexeme),
                },
        }
    }
}
