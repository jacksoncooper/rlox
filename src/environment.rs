use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::object::Object;

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

pub fn define(local: &mut Environment, name: &str, value: &Object) {
    local.borrow_mut().values.insert(name.to_string(), Object::clone(value));
}

pub fn assign(local: &mut Environment, name: &str, value: &Object) -> bool {
    if local.borrow().values.contains_key(name) {
        local.borrow_mut().values.insert(name.to_string(), Object::clone(value));
        true
    }
    else {
        match local.borrow_mut().enclosing {
            Some(ref mut enclosing) => assign(enclosing, name, value),
            None => false,
        }
    }
}

pub fn get(local: &Environment, name: &str) -> Option<Object> {
    match local.borrow().values.get(name) {
        Some(object) => Some(Object::clone(object)),
        None => match local.borrow().enclosing {
            Some(ref enclosing) => get(enclosing, name),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_in_enclosing() {
        let identifier = Token {
            token_type: TT::Identifier("waffle".to_string()),
            lexeme: "waffle".to_string(),
            line: 1
        };

        let value = Object::Number(4 as f64);

        let mut local = new();
        let mut enclosing = new();

        define(&mut enclosing, &identifier, &value);

        assert_eq!(get(&enclosing, &identifier).unwrap(), value);

        link(&mut local, &enclosing);

        assert_eq!(get(&local, &identifier).unwrap(), value);

        Ok(())
    }
}
