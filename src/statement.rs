use std::fmt;

use crate::expression::Expr;
use crate::token::Token;

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
                let mut readable = String::from("(block");
                for statement in statements {
                    readable.push(' ');
                    readable.push_str(statement.to_string().as_str());
                }
                readable.push(')');
                write!(f, "{}", readable)
            },
            Stmt::Expression { expression } =>
                write!(f, "(expr {})", expression),
            Stmt::Print { expression } =>
                write!(f, "(print {})", expression),
            Stmt::Var { name, initializer } =>
                match initializer {
                    Some(initializer) =>
                        write!(f, "(decl {} {})", name.lexeme, initializer),
                    None =>
                        write!(f, "(decl {})", name.lexeme),
                },
        }
    }
}
