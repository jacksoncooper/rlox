use std::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::callable::definitions as def;
use crate::object::Object;

type Bindings = HashMap<String, Object>;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    definition: def::Class,
    fields: Rc<RefCell<Bindings>>
}

impl Instance {
    pub fn new(definition: def::Class) -> Instance {
        Instance {
            definition,
            fields: Rc::new(RefCell::new(HashMap::new()))
        }
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Instance { definition: def::Class(name, ..), .. } = self;
        write!(f, "{} instance", name.to_name().1)
    }
}

impl Instance {
    pub fn get(&self, name: &str) -> Option<Object> {
        self.fields.borrow().get(name).map(Object::clone)
    }

    pub fn set(&mut self, name: &str, object: &Object) {
        self.fields.borrow_mut().insert(
            name.to_string(),
            object.clone()
        );
    }
}
