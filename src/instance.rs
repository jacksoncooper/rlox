use std::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::callable::Class;
use crate::object::Object;

type Fields = HashMap<String, Object>;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    class: Class,
    fields: Rc<RefCell<Fields>>
}

impl Instance {
    pub fn new(class: Class) -> Instance {
        Instance {
            class,
            fields: Rc::new(RefCell::new(HashMap::new()))
        }
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

impl Instance {
    pub fn get(&self, name: &str) -> Option<Object> {
        self.fields.borrow().get(name).map_or_else(
            || self.class.find_method(name).map(
                |function| Object::Callable(function.erase())
            ),
            |field| Some(Object::clone(field))
        )
    }

    pub fn set(&mut self, name: &str, object: &Object) {
        self.fields.borrow_mut().insert(
            name.to_string(),
            object.clone()
        );
    }
}
