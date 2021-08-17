use std::fmt;

use crate::parser::expression::Expr;
use crate::scanner::token::Token;

#[derive(Debug)]

pub enum Stmt {
    Block { statements: Vec<Stmt> },
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Option<Expr> },
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Block { statements } => {
                let mut readable: String = String::from("(block");
                for statement in statements {
                    readable.push(' ');
                    readable.push_str(&statement.to_string());
                }
                readable.push(')');
                write!(f, "{}", readable)
            },
            Stmt::Expression { expression } =>
                write!(f, "(expr {})", expression.to_string()),
            Stmt::Print { expression } =>
                write!(f, "(print {})", expression.to_string()),
            Stmt::Var { name, initializer } =>
                match initializer {
                    Some(initializer) =>
                        write!(f, "(decl {} {})", name.lexeme, initializer.to_string()),
                    None =>
                        write!(f, "(decl {})", name.lexeme),
                },
        }
    }
}
