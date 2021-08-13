use std::fmt;

#[derive(Debug)]

pub enum Object {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let readable: String = match self {
            Object::Nil => "nil".to_string(),
            Object::Boolean(bool)  => bool.to_string(),
            Object::Number(float)  => float.to_string(),
            Object::String(string) => format!("{:?}", string),
        };

        write!(f, "{}", readable)
    }
}
