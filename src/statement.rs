use std::rc::Rc;

use crate::expression::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Rc<Token>, Rc<Vec<Token>>, Rc<Vec<Stmt>>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Option<Expr>),
    While(Expr, Box<Stmt>),
}
