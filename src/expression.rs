use crate::object::Object;
use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    Assignment(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Get(Box<Expr>, Token),
    Grouping(Box<Expr>),
    Literal(Object),
    Logical(Box<Expr>, Token, Box<Expr>),
    Set(Box<Expr>, Token, Box<Expr>),
    Super(Token, Token),
    This(Token),
    Unary(Token, Box<Expr>),
    Variable(Token),
}

pub trait Visitor<T> {
    fn visit_assignment(&mut self, name: &Token, object: &Expr) -> T;
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> T;
    fn visit_get(&mut self, object: &Expr, name: &Token) -> T;
    fn visit_grouping(&mut self, expression: &Expr) -> T;
    fn visit_literal(&mut self, object: &Object) -> T;
    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> T;
    fn visit_super(&mut self, keyword: &Token, method: &Token) -> T;
    fn visit_this(&mut self, object: &Token) -> T;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_variable(&mut self, name: &Token) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Expr::Assignment(name, object) =>
                visitor.visit_assignment(name, object),
            Expr::Binary(left, operator, right) =>
                visitor.visit_binary(left, operator, right),
            Expr::Call(callee, paren, arguments) =>
                visitor.visit_call(callee, paren, arguments),
            Expr::Get(object, name) =>
                visitor.visit_get(object, name),
            Expr::Grouping(expression) =>
                visitor.visit_grouping(expression),
            Expr::Literal(object) =>
                visitor.visit_literal(object),
            Expr::Logical(left, operator, right) =>
                visitor.visit_logical(left, operator, right),
            Expr::Set(object, name, value) =>
                visitor.visit_set(object, name, value),
            Expr::Super(keyword, method) =>
                visitor.visit_super(keyword, method),
            Expr::This(object) =>
                visitor.visit_this(object),
            Expr::Unary(operator, right) =>
                visitor.visit_unary(operator, right),
            Expr::Variable(name) =>
                visitor.visit_variable(name),
        }
    }
}
