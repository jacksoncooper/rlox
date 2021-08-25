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
    Function {
        name: Rc<Token>,
        parameters: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
        closure: env::Environment,
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Callable::Clock, Callable::Clock) => true,
            (Callable::Function { name, .. },
             Callable::Function { name: other_name, .. }) =>
                name == other_name, // [1]
            _ => false,
        }
    }
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Clock => 0,
            Callable::Function { parameters, .. } => {
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
            Callable::Function { parameters, body, closure, .. } => {
                let mut local = env::new_with_enclosing(closure);

                for (parameter, argument) in parameters.iter().zip(&arguments) {
                    env::define(&mut local, parameter.to_name(), argument);
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
            Callable::Clock => write!(f, "<native fn>"),
            Callable::Function { name, .. } =>
                write!(f, "<fn {}>", name.to_name())
        }
    }
}

// [1]

 // Compare the identifier Token for equality. This contains the line on which
 // the function is defined. If two functions with the same name are defined on
 // the same line, the second definition will replace the first.
