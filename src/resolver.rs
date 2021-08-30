use std::collections::HashMap;

use crate::callable::definitions as def;
use crate::error;
use crate::expression::{self as expr, Expr};
use crate::object::Object;
use crate::statement::{self as stmt, Stmt};
use crate::token::Token;

#[derive(Clone, Copy, PartialEq)]
enum Function {
    Global,
    Function,
    Method,
    Initializer,
}

#[derive(Clone, Copy, PartialEq)]
enum Class {
    Global,
    Class,
    Subclass,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    resolutions: HashMap<usize, usize>,
    function_scope: Function,
    class_scope: Class,
    stumbled: bool,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: Vec::new(),
            resolutions: HashMap::new(),
            function_scope: Function::Global,
            class_scope: Class::Global,
            stumbled: false,
        }
    }

    pub fn consume(self) -> Result<HashMap<usize, usize>, error::LoxError> {
        if self.stumbled {
            Err(error::LoxError::Resolve)
        } else {
            Ok(self.resolutions)
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_expression(&mut self, expression: &Expr) {
        expression.accept(self)
    }

    fn resolve_statement(&mut self, statement: &Stmt) {
        statement.accept(self)
    }

    fn resolve_function(
        &mut self, definition: &def::Function,
        function_scope: Function,
    ) {
        let def::Function(_, parameters, body) = definition;
        let parameters: &Vec<Token> = parameters;
        let enclosing_function = self.function_scope;

        self.begin_scope();

        self.function_scope = function_scope;

        for parameter in parameters {
            // TODO: It's not technically necessary to declare and define the
            // parameter name. Just a definition would suffice but the
            // semantics are nice and Bob does it in the book. We can afford
            // a redundant hash and flipping a boolean.

            self.declare(parameter);
            self.define(parameter);
        }

        self.resolve_statements(body);

        self.end_scope();

        self.function_scope = enclosing_function;
    }

    fn resolve_local(&mut self, name: &Token) {
        let (identifier, name) = name.to_name();

        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.resolutions.insert(*identifier, depth);
                return;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last() {
            if scope.contains_key(name.to_name().1) {
                self.stumble(name, "Already a variable with this name in this scope.");
            } else {
                self.add_to_scope(name.to_name().1, false);
            }
        }
    }

    fn define(&mut self, name: &Token) {
        self.add_to_scope(name.to_name().1, true)
    }

    fn add_to_scope(&mut self, name: &str, resolved: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), resolved);
        }
    }

    fn stumble(&mut self, at: &Token, reason: &str) {
        error::parse_error(at, reason);
        self.stumbled = true;
    }
}

impl expr::Visitor<()> for Resolver {
    fn visit_assignment(&mut self, name: &Token, object: &Expr) {
        self.resolve_expression(object);
        self.resolve_local(name);
    }

    fn visit_binary(&mut self, left: &Expr, _: &Token, right: &Expr) {
        self.resolve_expression(left);
        self.resolve_expression(right);
    }

    fn visit_call(&mut self, callee: &Expr, _: &Token, arguments: &[Expr]) {
        self.resolve_expression(callee);

        for argument in arguments {
            self.resolve_expression(argument);
        }
    }

    fn visit_get(&mut self, object: &Expr, _: &Token) {
        self.resolve_expression(object);
    }

    fn visit_grouping(&mut self, expression: &Expr) {
        self.resolve_expression(expression);
    }

    fn visit_literal(&mut self, _: &Object) { }

    fn visit_logical(&mut self, left: &Expr, _: &Token, right: &Expr) {
        self.resolve_expression(left);
        self.resolve_expression(right);
    }

    fn visit_set(&mut self, object: &Expr, _: &Token, value: &Expr) {
        self.resolve_expression(value);
        self.resolve_expression(object);
    }

    fn visit_super(&mut self, keyword: &Token, _: &Token) {
        if self.class_scope == Class::Global {
            self.stumble(keyword, "Can't use 'super' outside of a class.");
        } else if self.class_scope == Class::Class {
            self.stumble(keyword, "Can't use 'super' in a class with no superclass.");
        }

        self.resolve_local(keyword);
    }

    fn visit_this(&mut self, this: &Token) {
        if self.class_scope == Class::Global {
            self.stumble(this, "Can't use 'this' outside of a class.");
        }

        self.resolve_local(this);
    }

    fn visit_unary(&mut self, _: &Token, right: &Expr) {
        self.resolve_expression(right);
    }

    fn visit_variable(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(name.to_name().1) {
                self.stumble(name, "Can't read local variable in its own initializer.");
            }

            self.resolve_local(name);
        }
    }
}

impl stmt::Visitor<()> for Resolver {
    fn visit_block(&mut self, statements: &[Stmt]) {
        self.begin_scope();
        self.resolve_statements(statements);
        self.end_scope();
    }

    fn visit_class(&mut self, definition: &def::Class) {
        let def::Class(name, parent, methods) = definition;

        let enclosing_class = self.class_scope;
        self.class_scope = Class::Class;

        self.declare(name);
        self.define(name);

        if let Some(parent) = parent {
            self.class_scope = Class::Subclass;

            if name.to_name().1 == parent.to_name().1 {
                self.stumble(parent, "A class can't inherit from itself.");
            }

            self.resolve_local(parent);
        }

        if parent.is_some() {
            self.begin_scope();
            self.add_to_scope("super", true);
        }

        self.begin_scope();

        self.add_to_scope("this", true);

        for method in methods {
            let def::Function(name, ..) = method;

            let scope = if name.to_name().1 == "init" {
                Function::Initializer
            } else { Function::Method };

            self.resolve_function(method, scope);
        }

        self.end_scope();

        if parent.is_some() { self.end_scope(); }

        self.class_scope = enclosing_class;
    }

    fn visit_expression(&mut self, expression: &Expr) {
        self.resolve_expression(expression)
    }

    fn visit_function(&mut self, definition: &def::Function) {
        let def::Function(name, ..) = definition;

        self.declare(name);
        self.define(name);

        self.resolve_function(definition, Function::Function);
    }

    fn visit_if(
        &mut self, condition: &Expr,
        then_branch: &Stmt, else_branch: &Option<Box<Stmt>>
    ) {
        self.resolve_expression(condition);
        self.resolve_statement(then_branch);

        if let Some(statement) = else_branch {
            self.resolve_statement(statement);
        }
    }

    fn visit_print(&mut self, object: &Expr) {
        self.resolve_expression(object);
    }

    fn visit_return(&mut self, keyword: &Token, object: &Option<Expr>) {
        if self.function_scope == Function::Global {
            self.stumble(keyword, "Can't return from top-level code.");
        }
        
        if let Some(object) = object {
            if self.function_scope == Function::Initializer {
                self.stumble(keyword, "Can't return a value from an initializer.");
            }

            self.resolve_expression(object);
        }
    }

    fn visit_var(&mut self, name: &Token, object: &Option<Expr>) {
        self.declare(name);

        if let Some(object) = object {
            self.resolve_expression(object);
        }

        self.define(name);
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) {
        self.resolve_expression(condition);
        self.resolve_statement(body);
    }
}
