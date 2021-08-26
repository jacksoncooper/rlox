use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use crate::environment as env;
use crate::interpreter as int;
use crate::object::Object;
use crate::statement::Stmt;
use crate::token::Token;

// TODO: Deriving PartialEq to compare functions is hilariously slow. Among the
// other members, Rust compares their closures, recursively walking each
// environment and comparing their bindings. Learn how to compare memory
// addresses.

#[derive(Clone, Debug, PartialEq)]
pub enum Callable {
    Clock,
    Function(Rc<Token>, Rc<Vec<Token>>, Rc<Vec<Stmt>>, env::Environment)
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callable::Clock => write!(f, "<native fn>"),
            Callable::Function(name, ..) =>
                write!(f, "<fn {}>", name.to_name().1)
        }
    }
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Clock => 0,
            Callable::Function(_, parameters, ..) => {
                if parameters.len() <= 255 {
                    return parameters.len() as u8
                }

                // A panic here indicates a failure in the parser.
                panic!("more than 255 parameters");
            }
        }
    }

    pub fn call(
        &self,
        interpreter: &mut int::Interpreter,
        arguments: Vec<Object>
    ) -> Result<Object, int::Unwind> {
        match self {
            Callable::Clock => {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH);

                Ok(now.map_or_else(
                    |_| Object::Nil,
                    |t| Object::Number(Rc::new(t.as_secs_f64()))
                ))
            },
            Callable::Function(_, parameters, body, closure) => {
                let mut local = env::new_with_enclosing(closure);

                for (parameter, argument) in parameters.iter().zip(&arguments) {
                    env::define(&mut local, parameter.to_name().1, argument);
                }

                interpreter.execute_block(body, env::copy(&local))?;

                Ok(Object::Nil)
            }
        }
    }
}
