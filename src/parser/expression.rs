use crate::interpreter::object::Object;
use crate::scanner::token::Token;

#[derive(Debug)]

pub enum Expr {
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Grouping { grouping: Box<Expr> },
    Literal { value: Object },
    Unary { operator: Token, right: Box<Expr> },
}
