use std::cmp;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use crate::environment as env;
use crate::instance::Instance;
use crate::interpreter::{self as int, Interpreter};
use crate::object::Object;
use crate::token::Token;

use definitions as def;

pub mod definitions {
    use std::rc::Rc;

    use crate::statement::Stmt;
    use crate::token::Token;

    #[derive(Clone, Debug)]
    pub struct Class(pub Rc<Token>, pub Vec<Function>);

    #[derive(Clone, Debug)]
    pub struct Function(pub Rc<Token>, pub Rc<Vec<Token>>, pub Rc<Vec<Stmt>>);
}

type Methods = HashMap<String, Function>;

#[derive(Clone, Debug)]
pub struct Class(Rc<Token>, Rc<Methods>);

impl Class {
    pub fn new(token: Rc<Token>, methods: Rc<Methods>) -> Class {
        Class(token, methods)
    }

    pub fn erase(self) -> Callable {
        Callable::Class(self)
    }

    pub fn arity(&self) -> u8 {
        0
    }

    pub fn call(&self) -> Result<Object, int::Unwind> {
        Ok(Object::Instance(Instance::new(
            self.clone()
        )))
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        let Class(_, methods) = self;
        methods.get(name).map(Function::clone)
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Class(name, _) = self;
        write!(f, "{}", name.to_name().1)
    }
}

impl cmp::PartialEq for Class {
    fn eq(&self, other: &Class) -> bool {
        let Class(name, ..) = self;
        let Class(other_name, ..) = other;
        name.token_type == other_name.token_type
    }
}

#[derive(Clone, Debug)]
pub struct Function(def::Function, env::Environment);

impl Function {
    pub fn new(def: def::Function, env: env::Environment) -> Function {
        Function(def, env)
    }

    pub fn erase(self) -> Callable {
        Callable::Function(self)
    }

    pub fn arity(&self) -> u8 {
        let Function(def::Function(_, parameters, ..), _) = self;

        // TODO: This parameter check doesn't need to happen every time a
        // function is called. It can be done in the interpreter when visiting a
        // function definition. The problem is that a callable is a parasite
        // hooked into the syntax tree and shares its representation of function
        // parameters. I'd have to allocate them somewhere else.

        u8::try_from(parameters.len()).unwrap_or_else(
            // A panic here indicates a error in the parser.
            |_| panic!("more than 255 parameters")
        )
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, int::Unwind> {
        let Function(def::Function(_, parameters, body), closure) = self;

        let mut local = env::new_with_enclosing(closure);

        for (parameter, argument) in parameters.iter().zip(&arguments) {
            env::define(&mut local, parameter.to_name().1, argument);
        }

        interpreter.execute_block(body, env::copy(&local))?;

        Ok(Object::Nil)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Function(def::Function(name, ..), _) = self;
        write!(f, "<fn {}>", name.to_name().1)
    }
}

impl cmp::PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        let Function(def::Function(name, ..), _) = self;
        let Function(def::Function(other_name, ..), _) = other;
        name.token_type == other_name.token_type
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Native {
    Clock
}

impl Native {
    pub fn erase(self) -> Callable {
        Callable::Native(self)
    }

    pub fn arity(&self) -> u8 {
        match self {
            Native::Clock => 0
        }
    }

    pub fn call(
        &self,
        _: &Interpreter,
        _: Vec<Object>,
    ) -> Result<Object, int::Unwind> {
        match self {
            Native::Clock => call_clock()
        }
    }
}

fn call_clock() -> Result<Object, int::Unwind> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH);

    Ok(now.map_or_else(
        |_| Object::Nil,
        |t| Object::Number(Rc::new(t.as_secs_f64()))
    ))
}

impl fmt::Display for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Callable {
    Class(Class),
    Function(Function),
    Native(Native),
}

impl Callable {
    pub fn arity(&self) -> u8 {
        match self {
            Callable::Class(class) => class.arity(),
            Callable::Native(native) => native.arity(),
            Callable::Function(function) => function.arity(),
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Object>
    ) -> Result<Object, int::Unwind> {
        match self {
            Callable::Class(class) => class.call(),
            Callable::Function(function) =>
                function.call(interpreter, arguments),
            Callable::Native(native) =>
                native.call(interpreter, arguments),
        }
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let readable = match self {
            Callable::Class(class) => class.to_string(),
            Callable::Function(function) => function.to_string(),
            Callable::Native(native) => native.to_string(),
        };

        write!(f, "{}", readable)
    }
}
