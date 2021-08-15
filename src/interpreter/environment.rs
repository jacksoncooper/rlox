use std::collections::HashMap;

use crate::interpreter::Error;
use crate::interpreter::object::Object;
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    fn new() -> Environment {
        Environment { values: HashMap::new() }
    }

    fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    fn get(&self, token: &Token) -> Result<Object, Error> {
        match &token.token_type {
            TT::Identifier(name) =>
                match self.values.get(name) {
                    Some(object) => Ok(Object::clone(object)),
                    None => Err(Error {
                        token: Token::clone(token),
                        message: format!("Undefined variable '{}'.", name)
                    }),
                }

            // TODO: This pattern is becoming a problem. The parser should be
            // narrowing Token types like it does for Lox objects so that
            // partial functions don't litter the rest of the interpreter.
            _ => panic!("not an identifier")
        }
    }
}
