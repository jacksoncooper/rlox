use std::fmt;

use crate::callable::Callable;

#[derive(Clone, Debug, PartialEq)]

pub enum Object {
    Boolean(bool),
    Callable(Callable),
    Nil,
    Number(f64),
    String(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Boolean(bool)  => write!(f, "{}", bool),
            Object::Callable(_) => write!(f, "<function>"),
            Object::Nil => write!(f, "nil"),
            Object::Number(float)  => write!(f, "{}", float),
            Object::String(string) => write!(f, "{}", string),
        }
    }
}
