use crate::token::Token;

pub enum Expr {
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
    Grouping { grouping: Box<Expr> },
    Literal { value: Token },
    Unary { operator: Token, right: Box<Expr> },
}
