use crate::environment as env;
use crate::error;
use crate::expression::Expr;
use crate::object::Object;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token_type::TokenType as TT;

#[derive(Debug)]

pub struct Error {
    token: Token,
    message: String,
}

impl Error {
    pub fn new(token: &Token, message: &str) -> Error {
        Error {
            token: Token::clone(token),
            message: message.to_string()
        }
    }
}

pub struct Interpreter {
    local: env::Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { local: env::new() }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), error::LoxError> {
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                error::runtime_error(&error.token, &error.message);
                // A runtime error kills the interpreter.
                return Err(error::LoxError::InterpretError);
            }
        }

        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Block { statements } =>
                self.execute_block(statements),
            Stmt::Expression { expression } =>
                self.execute_expression(expression),
            Stmt::Print { expression } =>
                self.execute_print(expression),
            Stmt::Var { name, initializer } =>
                self.execute_variable_declaration(name, initializer),
        }
    }

    fn execute_block(&mut self, statements: Vec<Stmt>) -> Result<(), Error> {
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

    fn execute_expression(&mut self, expr: Expr) -> Result<(), Error>{
        self.evaluate(expr)?;
        Ok(())
    }

    fn execute_print(&mut self, expr: Expr) -> Result<(), Error>{
        let value: Object = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn execute_variable_declaration(&mut self, token: Token, initializer: Option<Expr>) -> Result<(), Error> {
        let value: Object = match initializer {
            Some(initializer) => {
                let value: Object = self.evaluate(initializer)?;
                value
            },
            None => Object::Nil,
        };

        env::define(&mut self.local, &token, &value);

        Ok(())
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Object, Error> {
        // This function gobbles the syntax tree intentionally to emphasize
        // that it is "reduced" to a value, or consumed.

        match expr {
            Expr::Assignment { name, value } =>
                self.evaluate_assignment(name, *value),
            Expr::Binary { left, operator, right } =>
                self.evaluate_binary(*left, operator, *right),
            Expr::Grouping { grouping } =>
                self.evaluate(*grouping),
            Expr::Literal { value } =>
                Ok(value),
            Expr::Unary { operator, right } =>
                self.evaluate_unary(operator, *right),
            Expr::Variable { name } =>
                // This one has a side effect, so we need to pass it &mut self.
                self.evaluate_variable(name),
        }
    }

    fn evaluate_assignment(&mut self, name: Token, value: Expr) -> Result<Object, Error> {
        let value: Object = self.evaluate(value)?;
        env::assign(&mut self.local, &name, &value)?;
        Ok(value)
    }

    fn evaluate_binary(&mut self, left: Expr, operator: Token, right: Expr) -> Result<Object, Error> {
        let left: Object = self.evaluate(left)?;
        let right: Object = self.evaluate(right)?;

        match operator.token_type {
            TT::BangEqual =>
                Ok(Object::Boolean(left != right)),
            TT::EqualEqual =>
                Ok(Object::Boolean(left == right)),
            TT::Greater =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left > right)),
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },
            TT::GreaterEqual =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left >= right)),
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },
            TT::Less =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left < right)),
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },
            TT::LessEqual =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Boolean(left <= right)),
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },
            TT::Minus =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(left - right)),
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },
            TT::Plus =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(left + right)),
                    (Object::String(mut left), Object::String(right)) => {
                        left.push_str(&right);
                        Ok(Object::String(left))
                    }
                    _ => Err(Error::new(&operator, "Operands must be two numbers or two strings.")),
                }
            TT::Slash =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        if right != 0 as f64 { Ok(Object::Number(left / right)) }
                        else { Err(Error::new(&operator, "Division by zero.")) }
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },
            TT::Star =>
                match (left, right) {
                    (Object::Number(left), Object::Number(right)) =>
                        Ok(Object::Number(left * right)),
                    _ => Err(Error::new(&operator, "Operands must be numbers.")),
                },

            // A panic here indicates an error in the parser.
            _ => panic!("token is not a binary operator")
        }
    }

    fn evaluate_unary(&mut self, operator: Token, right: Expr) -> Result<Object, Error> {
        let right: Object = self.evaluate(right)?;

        match operator.token_type {
            TT::Bang =>
                Ok(Object::Boolean(!is_truthy(right))),
            TT::Minus =>
                match right {
                    Object::Number(float) => Ok(Object::Number(-float)),
                    _ => Err(Error::new(&operator, "Operand must be a number.")),
                },
            
            // A panic here indicates an error in the parser. [1] 
            _ => panic!("token is not a unary operator")
        }
    }

    fn evaluate_variable(&self, token: Token) -> Result<Object, Error> {
        env::get(&self.local, &token)
    }
}

fn is_truthy(operand: Object) -> bool {
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
