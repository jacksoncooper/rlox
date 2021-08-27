use crate::callable::definitions as def;
use crate::expression::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class(def::Class),
    Expression(Expr),
    Function(def::Function),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Option<Expr>),
    While(Expr, Box<Stmt>),
}

pub trait Visitor<T> {
    fn visit_block(&mut self, statements: &[Stmt]) -> T;
    fn visit_class(&mut self, definition: &def::Class) -> T;
    fn visit_expression(&mut self, expression: &Expr) -> T;
    fn visit_function(&mut self, definition: &def::Function) -> T;
    fn visit_if(
        &mut self, condition: &Expr,
        then_branch: &Stmt, else_branch: &Option<Box<Stmt>>
    ) -> T;
    fn visit_print(&mut self, object: &Expr) -> T;
    fn visit_return(&mut self, keyword: &Token, object: &Expr) -> T;
    fn visit_var(&mut self, name: &Token, object: &Option<Expr>) -> T;
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Stmt::Block(statements) =>
                visitor.visit_block(statements),
            Stmt::Class(definition) =>
                visitor.visit_class(definition),
            Stmt::Expression(expression) =>
                visitor.visit_expression(expression),
            Stmt::Function(definition) =>
                visitor.visit_function(definition),
            Stmt::If(condition, then_branch, else_branch) =>
                visitor.visit_if(condition, then_branch, else_branch),
            Stmt::Print(object) =>
                visitor.visit_print(object),
            Stmt::Return(keyword, object) =>
                visitor.visit_return(keyword, object),
            Stmt::Var(name, object) =>
                visitor.visit_var(name, object),
            Stmt::While(condition, body) =>
                visitor.visit_while(condition, body),
        }
    }
}
