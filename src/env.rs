use std::collections::HashMap;
use std::rc::Rc;

use super::object::Object;

pub struct Env {
    table: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Self {
        let mut env = Env{table: HashMap::new()};
        env.init();
        env
    }

    pub fn set(&mut self, name: &str, value: Object) {
        self.table.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.table.get(name) {
            None => None,
            Some(v) => Some(Rc::clone(v)),
        }
    }
}
