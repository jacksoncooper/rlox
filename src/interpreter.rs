use crate::error;
use crate::interpreter::object::Object;
use crate::parser::expression::Expr;
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

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

type Computation = Result<Object, Error>;

pub fn show(expr: Expr) -> Result<String, error::LoxError> {
    match evaluate(expr) {
        Ok(object) => Ok(object.to_string()),
        Err(error) => {
            error::runtime_error(&error.token, &error.message);
            Err(error::LoxError::InterpretError)
        }
    }
}

fn evaluate(expr: Expr) -> Computation {
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
    }

}

fn evaluate_binary(left: Expr, operator: Token, right: Expr) -> Computation {
    let left: Object = evaluate(left)?;
    let right: Object = evaluate(right)?;
    let operands = (left, right);

    match operator.token_type {
        TT::Minus =>
            match operands {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Number(left + right)),
                _ => Err(Error::new(&operator, "Cannot add operands that are not both numbers.")),
            },

        // A panic here indicates an error in the parser.
        _ => panic!("token is not a binary operator")
    }
}

fn evaluate_unary(operator: Token, right: Expr) -> Computation {
    let operand: Object = evaluate(right)?;

    match operator.token_type {
        TT::Bang =>
            Ok(Object::Boolean(is_truthy(operand))),
        TT::Minus =>
            match operand {
                Object::Number(float) => Ok(Object::Number(-float)),
                _ => Err(Error::new(&operator, "Cannot negate an operand that isn't a number.")),
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

