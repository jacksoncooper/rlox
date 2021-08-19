use std::fmt;

use crate::expression::Expr;
use crate::token::Token;

#[derive(Debug)]

pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(Token, Option<Expr>),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Block(statements) => {
                let mut readable = String::from("(block");
                for statement in statements {
                    readable.push(' ');
                    readable.push_str(statement.to_string().as_str());
                }
                readable.push(')');
                write!(f, "{}", readable)
            },
            Stmt::Expression(expression) =>
                write!(f, "(expr {})", expression),
            Stmt::If(condition, then_branch, else_branch) =>
                match else_branch {
                    Some(else_branch) =>
                        write!(f, "(if {} {} {})", condition, then_branch, else_branch),
                    None =>
                        write!(f, "(if {} {})", condition, then_branch),
                }
            Stmt::Print(expression) =>
                write!(f, "(print {})", expression),
            Stmt::Var(name, initializer) =>
                match initializer {
                    Some(initializer) =>
                        write!(f, "(decl {} {})", name.lexeme, initializer),
                    None =>
                        write!(f, "(decl {})", name.lexeme),
                },
        }
    }
}
