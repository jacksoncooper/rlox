use crate::error;
use crate::interpreter::object::Object;
use crate::parser::expression::Expr;
use crate::parser::statement::Stmt;
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

pub mod environment;
pub mod object;

struct Error {
    token: Token,
    message: String,
}

impl Error {
    fn new(token: &Token, message: &str) -> Error {
        Error {
            token: Token::clone(token),
            message: message.to_string()
        }
    }
}

pub fn interpret(statements: Vec<Stmt>) -> Result<(), error::LoxError> {
    for statement in statements {
        if let Err(error) = execute(statement) {
            error::runtime_error(&error.token, &error.message);
            // A runtime error kills the interpreter.
            return Err(error::LoxError::InterpretError);
        }
    }

    Ok(())
}

fn execute(stmt: Stmt) -> Result<(), Error> {
    match stmt {
        Stmt::Expression { expression } =>
            execute_expression_statement(expression),
        Stmt::Print { expression } =>
            execute_print_statement(expression),
        Stmt::Var { name, initializer } =>
            unimplemented!()
    }
}

fn execute_expression_statement(expr: Expr) -> Result<(), Error>{
    evaluate(expr)?;
    Ok(())
}

fn execute_print_statement(expr: Expr) -> Result<(), Error>{
    let value: Object = evaluate(expr)?;
    println!("{}", value);
    Ok(())
}

fn evaluate(expr: Expr) -> Result<Object, Error> {
    // This function gobbles the syntax tree intentionally to emphasize that it
    // is "reduced" to a value, or consumed.

    match expr {
        Expr::Binary { left, operator, right } =>
            evaluate_binary(*left, operator, *right),
        Expr::Grouping { grouping } =>
            evaluate(*grouping),
        Expr::Literal { value } =>
            Ok(value),
        Expr::Unary { operator, right } =>
            evaluate_unary(operator, *right),
        Expr::Variable { name } =>
            unimplemented!()
    }

}

fn evaluate_binary(left: Expr, operator: Token, right: Expr) -> Result<Object, Error> {
    let left: Object = evaluate(left)?;
    let right: Object = evaluate(right)?;

    match operator.token_type {
        TT::BangEqual =>
            Ok(Object::Boolean(left != right)),
        TT::EqualEqual =>
            Ok(Object::Boolean(left == right)),
        TT::Greater =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Boolean(left > right)),
                _ => Err(Error::new(&operator, "Operands to (>) must be two numbers.")),
            },
        TT::GreaterEqual =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Boolean(left >= right)),
                _ => Err(Error::new(&operator, "Operands to (>=) must be two numbers.")),
            },
        TT::Less =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Boolean(left < right)),
                _ => Err(Error::new(&operator, "Operands to (<) must be two numbers.")),
            },
        TT::LessEqual =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Boolean(left <= right)),
                _ => Err(Error::new(&operator, "Operands to (<=) must be two numbers.")),
            },
        TT::Minus =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Number(left - right)),
                _ => Err(Error::new(&operator, "Operands to (-) must be two numbers.")),
            },
        TT::Plus =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Number(left + right)),
                (Object::String(mut left), Object::String(right)) => {
                    left.push_str(&right);
                    Ok(Object::String(left))
                }
                _ => Err(Error::new(&operator, "Operands to (+) must be two numbers or two strings.")),
            }
        TT::Slash =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    if right != 0 as f64 { Ok(Object::Number(left / right)) }
                    else { Err(Error::new(&operator, "Division by zero.")) }
                _ => Err(Error::new(&operator, "Operands to (/) must be two numbers.")),
            },
        TT::Star =>
            match (left, right) {
                (Object::Number(left), Object::Number(right)) =>
                    Ok(Object::Number(left * right)),
                _ => Err(Error::new(&operator, "Operands to (*) must be two numbers.")),
            },

        // A panic here indicates an error in the parser.
        _ => panic!("token is not a binary operator")
    }
}

fn evaluate_unary(operator: Token, right: Expr) -> Result<Object, Error> {
    let right: Object = evaluate(right)?;

    match operator.token_type {
        TT::Bang =>
            Ok(Object::Boolean(!is_truthy(right))),
        TT::Minus =>
            match right {
                Object::Number(float) => Ok(Object::Number(-float)),
                _ => Err(Error::new(&operator, "Operand to (-) must be a number.")),
            },
        
        // A panic here indicates an error in the parser. [1] 
        _ => panic!("token is not a unary operator")
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