use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::Error;
use crate::object::Object;
use crate::token::Token;
use crate::token_type::TokenType as TT;

// TODO:

// Don't use RefCell here. I tried to declare the enclosing field as an
// optional reference to a mutable environment, but the borrow checker got
// really upset, probably due to each environment in the chain having a
// lifetime as least as long as its parent, which makes the whole list persist
// until the head is dropped and causes overlapping references for reasons
// I don't fully understand.

// Because two block statements can never be evaluated in parallel, an explicit
// stack of environments would be much more performant. I'm using RefCell
// because I don't know to write a tree structure in Rust.

// I'll wait to implement this until I see what Bob has up his sleeve for
// closures.

// [1]

// This pattern is becoming a problem. The parser should be narrowing Token
// types like it does for Lox objects so that partial functions don't litter
// the interpreter.

pub type Environment = Rc<RefCell<Bindings>>;

pub struct Bindings {
    enclosing: Option<Environment>,
    values: HashMap<String, Object>,
}

pub fn new() -> Environment {
    Rc::new(RefCell::new(
        Bindings {
            enclosing: None,
            values: HashMap::new(),
        }
    ))
}

pub fn copy(local: &Environment) -> Environment {
    Rc::clone(local)
}

pub fn link(local: &mut Environment, enclosing: &Environment) {
    local.borrow_mut().enclosing = Some(Rc::clone(enclosing));
}

pub fn define(local: &mut Environment, token: &Token, value: &Object) {
    match token.token_type {
        TT::Identifier(ref name) => {
            local.borrow_mut().values.insert(
                String::clone(name),
                Object::clone(value)
            );
        },

        _ => panic!("not an identifier") // [1]
    }
}

pub fn assign(local: &mut Environment, token: &Token, value: &Object) -> Result<(), Error> {
    match token.token_type {
        TT::Identifier(ref name) =>
            if local.borrow().values.contains_key(name) {
                local.borrow_mut().values.insert(
                    String::clone(name),
                    Object::clone(value)
                );

                Ok(())
            }
            else {
                match local.borrow_mut().enclosing {
                    Some(ref mut enclosing) =>
                        assign(enclosing, token, value),
                    None =>
                        Err(Error::new(
                            token,
                            format!("Undefined variable '{}'.", name),
                        )),
                }
            }

        _ => panic!("not an identifier") // [1]
    }
}

pub fn get(local: &Environment, token: &Token) -> Result<Object, Error> {
    match token.token_type {
        TT::Identifier(ref name) =>
            match local.borrow().values.get(name) {
                Some(object) =>
                    Ok(Object::clone(object)),
                None =>
                    match local.borrow().enclosing {
                        Some(ref enclosing) =>
                            get(enclosing, token),
                        None =>
                            Err(Error::new(
                                token,
                                format!("Undefined variable '{}'.", name),
                            ))
                    }
            }

        _ => panic!("not an identifier") // [1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_in_enclosing() -> Result<(), Error> {
        let identifier = Token {
            token_type: TT::Identifier("waffle".to_string()),
            lexeme: "waffle".to_string(),
            line: 1
        };

        let value = Object::Number(4 as f64);

        let mut local = new();
        let mut enclosing = new();

        define(&mut enclosing, &identifier, &value);

        assert_eq!(get(&enclosing, &identifier)?, value);

        link(&mut local, &enclosing);

        assert_eq!(get(&local, &identifier)?, value);

        Ok(())
    }
}
