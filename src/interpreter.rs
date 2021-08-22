use crate::callable::Callable;
use crate::environment as env;
use crate::error;
use crate::expression::Expr;
use crate::object::Object;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token_type::TokenType as TT;

#[derive(Debug)]
struct Error {
    token: Token,
    message: String,
}

impl Error {
    pub fn new(token: &Token, message: String) -> Error {
        Error { token: Token::clone(token), message }
    }
}

pub struct Interpreter {
    local: env::Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = env::new();
        env::define(&mut globals, "clock", &Object::Callable(Callable::Clock));
        Interpreter { local: globals }
    }

    pub fn interpret(
        &mut self,
        statements: Vec<Stmt>
    ) -> Result<(), error::LoxError> {
        for statement in &statements {
            if let Err(error) = self.execute(statement) {
                error::runtime_error(&error.token, &error.message);
                // A runtime error kills the interpreter.
                return Err(error::LoxError::Interpret);
            }
        }

        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Block(statements) =>
                self.execute_block(statements),
            Stmt::Expression(expression) =>
                self.execute_expression(expression),
            Stmt::If(condition, then_branch, else_branch) =>
                self.execute_if(condition, then_branch, else_branch),
            Stmt::Print(value) =>
                self.execute_print(value),
            Stmt::Var(identifier, initializer) =>
                self.execute_variable_declaration(identifier, initializer),
            Stmt::While(condition, body) =>
                self.execute_while(condition, body),
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        let old_local = env::copy(&self.local);

        let mut new_local = env::new();
        env::link(&mut new_local, &old_local);

        self.local = new_local;

        for statement in statements {
            let status = self.execute(statement);

            // If the statement is in error, restore the previous environment
            // before bubbling the runtime error. There's no reason to do this
            // but it makes me feel nice.

            if status.is_err() { self.local = old_local; return status; }
        }
    
        self.local = old_local;

        Ok(())
    }

    fn execute_expression(&mut self, expr: &Expr) -> Result<(), Error>{
        self.evaluate(expr)?;
        Ok(())
    }

    fn execute_if(
        &mut self, condition: &Expr,
        then_branch: &Stmt, else_branch: &Option<Box<Stmt>>
    ) -> Result<(), Error> {
        let go_then = is_truthy(&self.evaluate(condition)?);
        
        if go_then {
            self.execute(then_branch)?;
        } else if let Some(statement) = else_branch {
            self.execute(statement)?;
        }

        Ok(())
    }

    fn execute_print(&mut self, expr: &Expr) -> Result<(), Error> {
        let value: Object = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn execute_variable_declaration(
        &mut self,
        identifier: &Token, initializer: &Option<Expr>
    ) -> Result<(), Error> {
        let name = to_name(identifier);

        let value: Object = match initializer {
            Some(initializer) => {
                let value: Object = self.evaluate(initializer)?;
                value
            },
            None => Object::Nil,
        };

        env::define(&mut self.local, name, &value);

        Ok(())
    }

    fn execute_while(&mut self, condition: &Expr, body: &Stmt) -> Result<(), Error> {
        while is_truthy(&self.evaluate(condition)?) {
            self.execute(body)?;
        }

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, Error> {
        // TODO: There's no need to produced an owned Object. The only mutation
        // occurs in the Environment type. The most glaring problem is cloning
        // Literal leaves to conform to the return type. Wrap objects in an Rc
        // maybe?

        match expr {
            Expr::Assignment(identifier, value) =>
                self.evaluate_assignment(identifier, value),
            Expr::Binary(left, operator, right) =>
                self.evaluate_binary(left, operator, right),
            Expr::Call(callee, paren, arguments) =>
                self.evaluate_call(callee, paren, arguments),
            Expr::Grouping(grouping) =>
                self.evaluate(grouping),
            Expr::Literal(value) =>
                Ok(Object::clone(value)),
            Expr::Logical(left, operator, right) =>
                self.evaluate_logical(left, operator, right),
            Expr::Unary(operator, right) =>
                self.evaluate_unary(operator, right),
            Expr::Variable(identifier) =>
                // This one has a side effect, so we need to pass it &mut self.
                self.evaluate_variable(identifier),
        }
    }

    fn evaluate_assignment(
        &mut self,
        identifier: &Token, value: &Expr
    ) -> Result<Object, Error> {
        let name = to_name(identifier);
        let value: Object = self.evaluate(value)?;

        if env::assign(&mut self.local, name, &value) {
            Ok(value)
        } else {
            Err(Error::new(
                identifier,
                format!("Undefined variable '{}'.", name)
            ))
        }
    }

    #[allow(clippy::float_cmp)]
    fn evaluate_binary(
        &mut self,
        left: &Expr, operator: &Token, right: &Expr
    ) -> Result<Object, Error> {
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
                        Err(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        )),
                },
            TT::GreaterEqual =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left >= right)),
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        )),
                },
            TT::Less =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left < right)),
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        )),
                },
            TT::LessEqual =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left <= right)),
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        )),
                },
            TT::Minus =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(left - right)),
                _ =>
                    Err(Error::new(
                        operator,
                        "Operands must be numbers.".to_string()
                    )),
                },
            TT::Plus =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(left + right)),
                    (Object::String(left), Object::String(right)) => {
                        let mut concatenation = String::new();
                        concatenation.push_str(&left);
                        concatenation.push_str(&right);
                        Ok(Object::String(concatenation))
                    },
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operands must be two numbers or two strings.".to_string(),
                        )),
                }
            TT::Slash =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        if right != 0 as f64 {
                            Ok(Object::Number(left / right))
                        } else {
                            Err(Error::new(
                                operator,
                                "Division by zero.".to_string()
                            ))
                        }
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operands must be numbers.".to_string()
                        )),
                },
            TT::Star =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(left * right)),
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                },

            // A panic here indicates an error in the parser.
            _ => panic!("token is not a binary operator")
        }
    }

    fn evaluate_call(
        &mut self,
        callee: &Expr, paren: &Token, arguments: &[Expr],
    ) -> Result<Object, Error> {
        let callee = self.evaluate(callee)?;

        return if let Object::Callable(callable) = callee {
            let mut objects = Vec::new();

            for argument in arguments {
                objects.push(self.evaluate(argument)?);
            }

            if arguments.len() != callable.arity() as usize {
                return Err(Error::new(
                    paren,
                    format!(
                        "Expected {} arguments but got {}.",
                        callable.arity(),
                        arguments.len()
                    )
                ));
            }

            Ok(callable.call(self, objects))
        } else {
            Err(Error::new(
                paren,
                "Can only call functions and classes.".to_string()
            ))
        }
    }

    fn evaluate_logical(
        &mut self,
        left: &Expr, operator: &Token, right: &Expr
    ) -> Result<Object, Error> {
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

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<Object, Error> {
        let right: Object = self.evaluate(right)?;

        match operator.token_type {
            TT::Bang =>
                Ok(Object::Boolean(!is_truthy(&right))),
            TT::Minus =>
                match right {
                    Object::Number(float) => Ok(Object::Number(-float)),
                    _ =>
                        Err(Error::new(
                            operator,
                            "Operand must be a number.".to_string()
                        )),
                },
            
            // A panic here indicates an error in the parser. [1] 
            _ => panic!("token is not a unary operator")
        }
    }

    fn evaluate_variable(&self, identifier: &Token) -> Result<Object, Error> {
        let name = to_name(identifier);

        if let Some(object) = env::get(&self.local, name) {
            Ok(object)
        } else {
            Err(Error::new(
                identifier,
                format!("Undefined variable '{}'.", name)
            ))
        }
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

fn to_name(token: &Token) -> &str {
    match token.token_type {
        TT::Identifier(ref name) => name,
        _ => panic!("token is not an identifier")
    }
}

// [1]

// In the future, it may be nice to break Token into separate, exhaustive types
// that correspond to the expressions they're used in. This would prevent the
// possibility of a panic! using the type system. But it would complicate the
// scanner and necessitate the use of trait objects. I think. I don't really
// know how to write Rust.
