use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::object::Object;

pub type Environment = Rc<RefCell<Bindings>>;

#[derive(Debug, PartialEq)]
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

pub fn new_with_enclosing(enclosing: &Environment) -> Environment {
    let mut new = new();
    link(&mut new, enclosing);
    new
}

pub fn copy(local: &Environment) -> Environment {
    Rc::clone(local)
}

pub fn link(local: &mut Environment, enclosing: &Environment) {
    let mut bindings = local.borrow_mut();
    bindings.enclosing = Some(Rc::clone(enclosing));
}

pub fn define(local: &mut Environment, name: &str, value: &Object) {
    let mut bindings = local.borrow_mut();
    bindings.values.insert(name.to_string(), Object::clone(value));
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

pub fn get_at(local: &Environment, distance: usize, name: &str) -> Object {
    let ancestor = ancestor(local, distance);
    let bindings = ancestor.borrow();

    match bindings.values.get(name) {
        Some(object) => Object::clone(object),

        // A panic here indicates an error in the resolver.
        None => panic!(
            "failed to find '{}' at distance {}", name, distance
        )
    }
}

pub fn assign(local: &mut Environment, name: &str, value: &Object) -> bool {
    if local.borrow().values.contains_key(name) {
        local.borrow_mut().values .insert(name.to_string(), Object::clone(value));
        true
    } else {
        match local.borrow_mut().enclosing {
            Some(ref mut enclosing) => assign(enclosing, name, value),
            None => false,
        }
    }
}

pub fn assign_at(local: &Environment, distance: usize, name: &str, object: &Object) {
    let ancestor = ancestor(local, distance);
    let mut bindings = ancestor.borrow_mut();
    bindings.values.insert(name.to_string(), Object::clone(object));
}

fn ancestor(local: &Environment, distance: usize) -> Environment {
    let mut current = copy(local);

    for _ in 1..=distance {
        let also_current = copy(&current);
        let bindings = also_current.borrow();
        let parent = &bindings.enclosing;

        match parent {
            Some(next) =>
                current = copy(next),

            // A panic here indicates an error in the resolver.
            None => panic!(
                "failed to step {} environments from the given scope",
                distance
            )
        }
    }

    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_in_enclosing() {
        let value = Object::Number(Rc::new(4 as f64));

        let mut local = new();
        let mut enclosing = new();

        define(&mut enclosing, "waffle", &value);

        assert_eq!(get(&enclosing, "waffle").unwrap(), value);

        link(&mut local, &enclosing);

        assert_eq!(get(&local, "waffle").unwrap(), value);
    }
}

// [1]

// This pattern is becoming a problem. The parser should be narrowing Token
// types like it does for Lox objects so that partial functions don't litter
// the interpreter.
