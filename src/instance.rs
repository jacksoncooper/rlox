use std::fmt;

use crate::callable::definitions as def;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance(def::Class);

impl Instance {
    pub fn new(definition: def::Class) -> Instance {
        Instance(definition)
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Instance(def::Class(name, ..)) = self;
        write!(f, "{} instance", name.to_name().1)
    }
}
