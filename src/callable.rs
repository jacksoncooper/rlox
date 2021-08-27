use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use crate::environment as env;
use crate::instance::Instance;
use crate::interpreter as int;
use crate::object::Object;

use definitions as def;

pub mod definitions {
    use std::cmp;
    use std::rc::Rc;

    use crate::statement::Stmt;
    use crate::token::Token;

    #[derive(Clone, Debug)]
    pub struct Class(pub Rc<Token>, pub Rc<Vec<Function>>);

    impl cmp::PartialEq for Class {
        fn eq(&self, other: &Class) -> bool {
            let Class(name, ..) = self;
            let Class(other_name, ..) = other;
            name.token_type == other_name.token_type
        }
    }

    #[derive(Clone, Debug)]
    pub struct Function(pub Rc<Token>, pub Rc<Vec<Token>>, pub Rc<Vec<Stmt>>);

    impl cmp::PartialEq for Function {
        fn eq(&self, other: &Function) -> bool {
            let Function(name, ..) = self;
            let Function(other_name, ..) = other;
            name.token_type == other_name.token_type
        }
    }
}

#[derive(Clone, Debug)]
pub enum Callable {
    Class(def::Class),
    Clock,
    Function(def::Function, env::Environment)
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callable::Class(def::Class(name, ..)) =>
                write!(f, "{}", name.to_name().1),
            Callable::Clock =>
                write!(f, "<native fn>"),
            Callable::Function(def::Function(name, ..), _) =>
                write!(f, "<fn {}>", name.to_name().1),
        }
    }
}

impl cmp::PartialEq for Callable {
    fn eq(&self, other: &Callable) -> bool {
        // Identifier tokens now contain a unique identifier produced
        // by the scanner. We implicitly compare those.
        match (self, other) {
            ( Callable::Class(definition)
            , Callable::Class(other_definition)
            ) => definition == other_definition,
            ( Callable::Function(definition, _)
            , Callable::Function(other_definition, _)
            ) => definition == other_definition,
            ( Callable::Clock
            , Callable::Clock
            ) => true,
            _ => false,
        }
    }
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Class(_) => 0,
            Callable::Clock => 0,
            Callable::Function(def::Function(_, parameters, ..), _) => {
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
            Callable::Class(definition) => {
                Ok(Object::Instance(
                    Instance::new(definition.clone())
                ))
            },
            Callable::Clock => {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH);

                Ok(now.map_or_else(
                    |_| Object::Nil,
                    |t| Object::Number(Rc::new(t.as_secs_f64()))
                ))
            },
            Callable::Function(def::Function(_, parameters, body), closure) => {
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
