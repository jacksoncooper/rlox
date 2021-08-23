use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use crate::environment as env;
use crate::interpreter as int;
use crate::object::Object;
use crate::statement::Stmt;
use crate::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum Callable {
    Clock,
    Function(Rc<Token>, Rc<Vec<Token>>, Rc<Vec<Stmt>>),
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Clock => 0,
            Callable::Function(_, parameters, _) => {
                if parameters.len() <= 255 {
                    return parameters.len() as u8
                }

                // A panic here indicates a failure in the interpreter.
                panic!("more than 255 parameters");
            }
        }
    }

    pub fn call(
        &self,
        interpreter: &mut int::Interpreter,
        arguments: Vec<Object>
    ) -> Result<Object, int::Error> {
        match self {
            Callable::Clock => {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH);

                Ok(now.map_or_else(
                    |_| Object::Nil,
                    |t| Object::Number(t.as_secs_f64())
                ))
            },
            Callable::Function(_, parameters, body) => {
                let mut local = env::new();
                env::link(&mut local, &interpreter.globals());

                for (parameter, argument) in parameters.iter().zip(&arguments) {
                    env::define(&mut local, int::to_name(parameter), argument);
                }

                interpreter.execute_block(body, env::copy(&local))?;

                Ok(Object::Nil)
            }
        }
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callable::Clock => write!(f, "<native function clock>"),
            Callable::Function(identifier, _, _) =>
                write!(f, "<function {}>", int::to_name(identifier))
        }
    }
}
