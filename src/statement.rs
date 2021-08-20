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
    While(Expr, Box<Stmt>),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Block(stmts) => {
                let mut readable = String::from("(block");
                for stmt in stmts {
                    readable.push(' ');
                    readable.push_str(stmt.to_string().as_str());
                }
                readable.push(')');
                write!(f, "{}", readable)
            },
            Stmt::Expression(expr) =>
                write!(f, "(expr {})", expr),
            Stmt::If(condition, then_branch, else_branch) =>
                match else_branch {
                    Some(else_branch) =>
                        write!(f, "(if {} {} {})", condition, then_branch, else_branch),
                    None =>
                        write!(f, "(if {} {})", condition, then_branch),
                }
            Stmt::Print(value) =>
                write!(f, "(print {})", value),
            Stmt::Var(name, initializer) =>
                match initializer {
                    Some(initializer) =>
                        write!(f, "(decl {} {})", name.lexeme, initializer),
                    None =>
                        write!(f, "(decl {})", name.lexeme),
                },
            Stmt::While(condition, body) =>
                write!(f, "(while {} {})", condition, body),
        }
    }
}
