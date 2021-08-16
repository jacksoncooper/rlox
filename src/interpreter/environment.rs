use std::collections::HashMap;

use super::Error;
use super::Object;

use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub(super) fn new() -> Environment {
        Environment { values: HashMap::new() }
    }

    // [1] This pattern is becoming a problem. The parser should be narrowing
    // Token types like it does for Lox objects so that partial functions don't
    // litter the rest of the interpreter.

    pub(super) fn define(&mut self, token: &Token, value: &Object) {
        match token.token_type {
            TT::Identifier(ref name) => {
                self.values.insert(String::clone(name), Object::clone(value));
            },

            _ => panic!("not an identifier") // [1]
        }
    }

    pub(super) fn assign(&mut self, token: &Token, value: &Object) -> Result<(), Error> {
        match token.token_type {
            TT::Identifier(ref name) =>
                if self.values.contains_key(name) {
                    self.values.insert(String::clone(name), Object::clone(value));
                    Ok(())
                } else {
                    Err(Error::new(token, &format!("Undefined variable '{}'.", name)))
                }

            _ => panic!("not an identifier") // [1]
        }
    }

    pub(super) fn get(&self, token: &Token) -> Result<Object, Error> {
        match token.token_type {
            TT::Identifier(ref name) =>
                match self.values.get(name) {
                    Some(object) => Ok(Object::clone(object)),
                    None => Err(Error::new(token, &format!("Undefined variable '{}'.", name))),
                }

            _ => panic!("not an identifier") // [1]
        }
    }
}
