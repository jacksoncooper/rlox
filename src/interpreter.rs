use std::rc::Rc;
use std::collections::HashMap;

use crate::callable::{self as call, definitions as def};
use crate::environment as env;
use crate::error;
use crate::expression::{self as expr, Expr};
use crate::object::Object;
use crate::statement::{self as stmt, Stmt};
use crate::token::Token;
use crate::token_type::TokenType as TT;

#[derive(Debug)]
pub struct Error {
    token: Token,
    message: String,
}

pub enum Unwind {
    Error(Error),
    Return(Token, Object),
}

impl Error {
    pub fn new(token: &Token, message: String) -> Error {
        Error { token: Token::clone(token), message }
    }
}

pub struct Interpreter {
    global: env::Environment,
    local: env::Environment,
    resolutions: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new(resolutions: HashMap<usize, usize>) -> Interpreter {
        let mut global = env::new();

        env::define(
            &mut global, "clock",
            &Object::Callable(call::Callable::Native(call::Native::Clock))
        );

        Interpreter {
            global: env::copy(&global),
            local: env::copy(&global),
            resolutions,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), error::LoxError> {
        for statement in &statements {
            if let Err(error) = self.execute(statement) {
                match error {
                    Unwind::Error(error) =>
                        error::runtime_error(&error.token, &error.message),
                    Unwind::Return(..) =>
                        // A panic here indicates an error in the resolver or interpreter.
                        panic!("uncaught return")
                }

                // A runtime error kills the interpreter.
                return Err(error::LoxError::Interpret);
            }
        }

        Ok(())
    }

    pub fn evaluate(&mut self, expression: &Expr) -> Result<Object, Unwind> {
        expression.accept(self)
    }

    pub fn execute(&mut self, statement: &Stmt) -> Result<(), Unwind> {
        statement.accept(self)
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt], new_local: env::Environment
    ) -> Result<(), Unwind> {
        let old_local = env::copy(&self.local);

        self.local = new_local;

        for statement in statements {
            let result = self.execute(statement);

            // If the statement is in error, restore the previous environment
            // before bubbling the runtime error. There's no reason to do this
            // but it makes me feel nice.

            if result.is_err() { self.local = old_local; return result; }
        }
   
        self.local = old_local;

        Ok(())
    }

    fn look_up_variable(&self, token: &Token) -> Result<Object, Unwind> {
        let (identifier, name) = token.to_name();

        match self.resolutions.get(identifier) {
            Some(distance) => Ok(env::get_at(&self.local, *distance, name)),
            None => env::get(&self.global, name).map_or_else(
                || Err(Unwind::Error(Error::new(
                    token, format!("Undefined variable '{}'.", name)
                ))),
                Ok
            ),
        }
    }
}

impl expr::Visitor<Result<Object, Unwind>> for Interpreter {
    fn visit_assignment(
        &mut self,
        token: &Token, object: &Expr
    ) -> Result<Object, Unwind> {
        let (identifier, name) = token.to_name();
        let object: Object = self.evaluate(object)?;

        match self.resolutions.get(identifier) {
            Some(distance) => {
                env::assign_at(&self.local, *distance, name, &object);
                Ok(object)
            },
            None =>
                if env::assign(&mut self.global, name, &object) {
                    Ok(object)
                } else {
                    Err(Unwind::Error(Error::new(
                        token, format!("Undefined variable '{}'.", name)
                    )))
                }
        }
    }

    #[allow(clippy::float_cmp)]
    fn visit_binary(
            &mut self,
            left: &Expr, operator: &Token, right: &Expr
    ) -> Result<Object, Unwind> {
        let left  = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TT::BangEqual =>
                Ok(Object::Boolean(left != right)),
            TT::EqualEqual =>
                Ok(Object::Boolean(left == right)),
            TT::Greater =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left > right)),
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        ))),
                },
            TT::GreaterEqual =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left >= right)),
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        ))),
                },
            TT::Less =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left < right)),
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        ))),
                },
            TT::LessEqual =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left <= right)),
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        ))),
                },
            TT::Minus =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(Rc::new(*left - *right))),
                _ =>
                    Err(Unwind::Error(Error::new(
                        operator,
                        "Operands must be numbers.".to_string()
                    ))),
                },
            TT::Plus =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(Rc::new(*left + *right))),
                    (Object::String(left), Object::String(right)) => {
                        let mut concatenation = String::new();
                        concatenation.push_str(&left);
                        concatenation.push_str(&right);
                        Ok(Object::String(Rc::new(concatenation)))
                    },
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be two numbers or two strings.".to_string(),
                        ))),
                }
            TT::Slash =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        if *right != 0 as f64 {
                            Ok(Object::Number(Rc::new(*left / *right)))
                        } else {
                            Err(Unwind::Error(Error::new(
                                operator,
                                "Division by zero.".to_string()
                            )))
                        }
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        ))),
                },
            TT::Star =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(Rc::new(*left * *right))),
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        ))),
                },

            // A panic here indicates an error in the parser.
            _ => panic!("token is not a binary operator")
        }
    }

    fn visit_call(
        &mut self,
        callee: &Expr, paren: &Token, arguments: &[Expr]
    ) -> Result<Object, Unwind> {
        let callee = self.evaluate(callee)?;

        return if let Object::Callable(callable) = callee {
            let mut objects = Vec::new();

            for argument in arguments {
                objects.push(self.evaluate(argument)?);
            }

            if arguments.len() > 255 {
                // A panic here indicates a error in the parser.
                panic!("more than 255 arguments");
            }

            if arguments.len() as u8 != callable.arity() {
                return Err(Unwind::Error(Error::new(
                    paren,
                    format!(
                        "Expected {} arguments but got {}.",
                        callable.arity(),
                        arguments.len()
                    )
                )));
            }

            let result = callable.call(self, objects);

            if let Err(Unwind::Return(_, object)) = result {
                return Ok(object);
            }

            result
        } else {
            Err(Unwind::Error(Error::new(
                paren,
                "Can only call functions and classes.".to_string()
            )))
        }
    }
 
    fn visit_get(&mut self, object: &Expr, token: &Token) -> Result<Object, Unwind> {
        let object = self.evaluate(object)?;
        let name = token.to_name().1;

        match object {
            Object::Instance(instance) =>
                instance.get(name).map_or_else(
                    || Err(Unwind::Error(Error::new(
                        token, format!("Undefined property '{}'.", name)
                    ))),
                    Ok
                ),
            _ => Err(Unwind::Error(Error::new(
                token, "Only instances have properties.".to_string()
            )))
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Result<Object, Unwind> {
        self.evaluate(expression)
    }

    fn visit_literal(&mut self, object: &Object) -> Result<Object, Unwind> {
        Ok(Object::clone(object))
    }

    fn visit_logical(
        &mut self,
        left: &Expr, operator: &Token, right: &Expr
    ) -> Result<Object, Unwind> {
        // Lox's logical operators are really ~~weird~~ fun. They are only
        // guaranteed to return a value with the truth value of the logical
        // expression. Combined short-circuiting evaluation, this makes them
        // deterministic.

        // T or  _ -> left operand
        // F or  _ -> right operand
        // T and _ -> right operand
        // F and _ -> left operand

        let left = self.evaluate(left)?;

        match operator.token_type {
            TT::Or => {
                if is_truthy(&left) { return Ok(left); }
            },
            TT::And => {
                if !is_truthy(&left) { return Ok(left); }
            },

            // A panic here indicates an error in the parser.
            _ => panic!("token is not a logical operator")
        }

        self.evaluate(right)
    }

    fn visit_set(&mut self, object: &Expr, token: &Token, value: &Expr) -> Result<Object, Unwind> {
        let object = self.evaluate(object)?;

        match object {
            Object::Instance(mut instance) => {
                let name = token.to_name().1;
                let value = self.evaluate(value)?;
                instance.set(name, &value);
                Ok(value)
            },
            _ => Err(Unwind::Error(Error::new(
                token,
                "Only instances have fields.".to_string()
            )))
        }
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Result<Object, Unwind> {
        let right: Object = self.evaluate(right)?;

        match operator.token_type {
            TT::Bang =>
                Ok(Object::Boolean(!is_truthy(&right))),
            TT::Minus =>
                match right {
                    Object::Number(float) => Ok(Object::Number(Rc::new(-*float))),
                    _ =>
                        Err(Unwind::Error(Error::new(
                            operator,
                            "Operand must be a number.".to_string()
                        ))),
                },
            
            // A panic here indicates an error in the parser. [1] 
            _ => panic!("token is not a unary operator")
        }
    }

    fn visit_variable(&mut self, name: &Token) -> Result<Object, Unwind> {
        self.look_up_variable(name)
    }
}

impl stmt::Visitor<Result<(), Unwind>> for Interpreter {
    fn visit_block(&mut self, statements: &[Stmt]) -> Result<(), Unwind> {
        self.execute_block(
            statements,
            env::new_with_enclosing(&self.local)
        )
    }

    fn visit_class(&mut self, definition: &def::Class) -> Result<(), Unwind> {
        let def::Class(token, function_definitions) = definition;
        let name = token.to_name().1;

        env::define(&mut self.local, name, &Object::Nil);

        let mut methods = HashMap::new();
        for function_definition in function_definitions {
            let def::Function(function_name, ..) = function_definition;
            methods.insert(
                function_name.to_name().1.to_string(),
                call::Function::new(
                    function_definition.clone(),
                    env::copy(&self.local)
                )
            );
        }

        let class = call::Class::new_callable(
            token.clone(),
            Rc::new(methods)
        );

        env::define(&mut self.local, name, &Object::Callable(class));

        Ok(())
    }

    fn visit_expression(&mut self, expression: &Expr) -> Result<(), Unwind> {
        self.evaluate(expression)?;
        Ok(())
    }

    fn visit_function(
        &mut self,
        definition: &def::Function
    ) -> Result<(), Unwind> {
        let def::Function(token, ..) = definition;
        let name = token.to_name().1;

        let object = call::Function::new_callable(
            definition.clone(),
            env::copy(&self.local)
        );

        env::define(&mut self.local, name, &Object::Callable(object));

        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>
    ) -> Result<(), Unwind> {
        let go_then = is_truthy(&self.evaluate(condition)?);
        
        if go_then {
            self.execute(then_branch)?;
        } else if let Some(statement) = else_branch {
            self.execute(statement)?;
        }

        Ok(())
    }

    fn visit_print(&mut self, object: &Expr) -> Result<(), Unwind> {
        let object: Object = self.evaluate(object)?;
        println!("{}", object);
        Ok(())
    }

    fn visit_return(&mut self, keyword: &Token, object: &Expr) -> Result<(), Unwind> {
        Err(Unwind::Return(
            Token::clone(keyword),
            self.evaluate(object)?
        ))
    }

    fn visit_var(&mut self, name: &Token, object: &Option<Expr>) -> Result<(), Unwind> {
        let name = name.to_name().1;

        let object: Object = match object {
            Some(initializer) => self.evaluate(initializer)?,
            None => Object::Nil,
        };

        env::define(&mut self.local, name, &object);

        Ok(())
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> Result<(), Unwind> {
        while is_truthy(&self.evaluate(condition)?) {
            self.execute(body)?;
        }

        Ok(())
    }
}

#[allow(clippy::match_like_matches_macro)]
fn is_truthy(operand: &Object) -> bool {
    // We're following Ruby because Ruby is pretty. 'false' and 'nil' are
    // falsey. Everything else is truthy.

    match operand {
        Object::Nil            => false,
        Object::Boolean(false) => false,
        _                      => true,
    }
}

// [1]

// In the future, it may be nice to break Token into separate, exhaustive types
// that correspond to the expressions they're used in. This would prevent the
// possibility of a panic! using the type system. But it would complicate the
// scanner and necessitate the use of trait objects. I think. I don't really
// know how to write Rust.
