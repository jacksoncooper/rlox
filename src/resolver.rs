use std::collections::HashMap;

use crate::error;
use crate::expression::{self as expr, Expr};
use crate::statement::{self as stmt, Stmt};
use crate::token::Token;

struct Resolver {
    scopes: Vec<HashMap<String, bool>>
}

impl Resolver {
    fn resolve_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_statement(&mut self, statement: &Stmt) {
        statement.accept(self)
    }

    fn resolve_expression(&mut self, expression: &Expr) {
        expression.accept(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        self.add_to_scope(name, false)
    }

    fn define(&mut self, name: &Token) {
        self.add_to_scope(name, true)
    }

    fn add_to_scope(&mut self, name: &Token, resolved: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_name().to_string(), resolved);
        }
    }
}

impl expr::Visitor<()> for Resolver {
    fn visit_variable(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(name.to_name()) {
                error::parse_error(
                    name,
                    "Can't read local variable in its own initializer."
                );
            }
        }

        self.resolve_local(name, name.to_name());
    }
}

impl stmt::Visitor<()> for Resolver {
    fn visit_block(&mut self, statements: &[Stmt]) {
        self.begin_scope();
        self.resolve_statements(statements);
        self.end_scope();
    }

    fn visit_var(&mut self, name: &Token, object: &Option<Expr>) {
        self.declare(name);

        if let Some(object) = object {
            self.resolve_expression(object);
        }

        self.define(name);
    }
}
