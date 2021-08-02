use crate::token::Token;
use crate::token_type;

pub enum Expr<'a> {
    Binary { left: Box<Expr<'a>>, operator: &'a Token, right: Box<Expr<'a>> },
    Grouping { grouping: Box<Expr<'a>> },
    Literal { value: &'a token_type::Literal },
    Unary { operator: &'a Token, right: Box<Expr<'a>> },
}
