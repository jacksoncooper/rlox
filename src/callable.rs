use std::cmp;
use std::convert::TryFrom;
use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use rustc_hash::FxHashMap;

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
    pub struct Class(pub Rc<Token>, pub Option<Rc<Token>>, pub Vec<Function>);

    #[derive(Clone, Debug)]
    pub struct Function(pub Rc<Token>, pub Rc<Vec<Token>>, pub Rc<Vec<Stmt>>);
}

type Methods = FxHashMap<String, Function>;

#[derive(Clone, Debug)]
pub struct Class(Rc<Token>, Option<Rc<Class>>, Rc<Methods>);

impl Class {
    pub fn new(
        name: Rc<Token>,
        parent: Option<Rc<Class>>,
        methods: Rc<Methods>
    ) -> Class {
        Class(name, parent, methods)
    }

    pub fn erase(self) -> Callable {
        Callable::Class(self)
    }

    pub fn arity(&self) -> u8 {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else { 0 }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Object>
    ) -> Result<Object, int::Unwind> {
        let instance = Instance::new(self.clone());

        if let Some(initializer) = self.find_method("init") {
            initializer.bind(&instance).call(interpreter, arguments)
        } else {
            Ok(Object::Instance(instance))
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        let Class(_, parent, methods) = self;

        methods.get(name).map_or_else(
            || parent.as_ref().and_then(|parent| parent.find_method(name)),
            |method| Some(method.clone())
        )
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Class(name, ..) = self;
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

// TODO: Yikes use record syntax for goodness' sake.

#[derive(Clone, Debug)]
pub struct Function(def::Function, env::Environment, bool);

impl Function {
    pub fn new(
        definition: def::Function,
        environment: env::Environment,
        is_initializer: bool
    ) -> Function {
        Function(definition, environment, is_initializer)
    }

    pub fn erase(self) -> Callable {
        Callable::Function(self)
    }

    pub fn arity(&self) -> u8 {
        let Function(def::Function(_, parameters, ..), ..) = self;

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
        let Function(
            def::Function(_, parameters, body),
            closure, is_initializer
        ) = self;

        let mut local = env::new_with_enclosing(closure);

        // A panic here indicates an error in the parser or interpreter.
        if parameters.len() != arguments.len() {
            panic!("zip destroys parameters or arguments")
        }

        for (parameter, argument) in parameters.iter().zip(&arguments) {
            env::define(&mut local, parameter.to_name().1, argument);
        }

        let result = interpreter.execute_block(body, env::copy(&local));

        match result {
            // The programmer returned with an explicit `return` keyword.
            Err(int::Unwind::Return(_, object)) =>
                if *is_initializer {
                    Ok(env::get_at(closure, 0, "this"))
                } else { Ok(object) },
            // Runtime error. Reconstruct its type to conform to Object.
            Err(error) => Err(error),
            // Implicit return, either `nil` or `this` if initializer.
            Ok(()) =>
                if *is_initializer {
                    Ok(env::get_at(closure, 0, "this"))
                } else { Ok(Object::Nil) },
        }
    }

    pub fn bind(&self, instance: &Instance) -> Function {
        let Function(definition, closure, is_initializer) = self;
        let mut with_this = env::new_with_enclosing(closure);
        env::define(&mut with_this, "this", &Object::Instance(instance.clone()));
        Function(definition.clone(), with_this, *is_initializer)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Function(def::Function(name, ..), ..) = self;
        write!(f, "<fn {}>", name.to_name().1)
    }
}

impl cmp::PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        let Function(def::Function(name, ..), closure, _) = self;
        let Function(def::Function(other_name, ..), other_closure, _) = other;

        name.token_type == other_name.token_type
            // This is a hack to make rlox behave like the reference Java
            // implementation. Bound methods are equal if they share the
            // same binding site, not the same object.
            && Rc::ptr_eq(closure, other_closure)
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
        |t| Object::Number(t.as_secs_f64())
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
            Callable::Class(class) =>
                class.call(interpreter, arguments),
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
