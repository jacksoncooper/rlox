use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use crate::environment as env;
use crate::interpreter as int;
use crate::object::Object;
use crate::statement::Stmt;
use crate::token::Token;

#[derive(Clone, Debug)]
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

impl cmp::PartialEq for Callable {
    fn eq(&self, other: &Callable) -> bool {
        match (self, other) {
            (Callable::Function(name, ..), Callable::Function(other_name, ..)) =>
                // Identifier tokens now contain a unique identifier produced
                // by the scanner. We implicitly compare those.
                name == other_name,
            (Callable::Clock, Callable::Clock) =>
                true,
            _ =>
                false,
        }
    }
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Clock => 0,
            Callable::Function(_, parameters, ..) => {
                // TODO: This parameter check doesn't need to happen every time
                // a function is called. It can be done in the interpreter
                // when visiting a function definition. The problem is that a
                // callable is a parasite hooked into the syntax tree and
                // shares its representation of function parameters. I'd have
                // to allocate them somewhere else.

                u8::try_from(parameters.len()).unwrap_or_else(
                    // A panic here indicates a error in the parser.
                    |_| panic!("more than 255 parameters")
                )
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
