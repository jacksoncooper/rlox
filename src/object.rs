use std::rc::Rc;
use std::fmt;

use crate::callable::Callable;

#[derive(Clone, Debug, PartialEq)]

pub enum Object {
    Boolean(bool),
    Callable(Callable),
    Nil,
    Number(Rc<f64>),
    String(Rc<String>),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Boolean(bool)  => write!(f, "{}", bool),
            Object::Callable(callable) => write!(f, "{}", callable),
            Object::Nil => write!(f, "nil"),
            Object::Number(float)  => write!(f, "{}", float),
            Object::String(string) => write!(f, "{}", string),
        }
    }
}
