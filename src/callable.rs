use std::time::SystemTime;

use crate::interpreter::Interpreter;
use crate::object::Object;

#[derive(Clone, Debug, PartialEq)]
pub enum Callable {
    Clock,
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Clock => 0,
        }
    }

    pub fn call(&self, _: &Interpreter, _: Vec<Object>) -> Object {
        match self {
            Callable::Clock => {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH);

                now.map_or_else(
                    |_| Object::Nil,
                    |t| Object::Number(t.as_secs_f64())
                )
            },
        }
    }
}
