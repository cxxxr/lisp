use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::object::Object;

pub struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    table: HashMap<String, Object>,
}

impl Env {
    pub fn new(parent: Option<Rc<RefCell<Env>>>) -> Self {
        Self {
            parent,
            table: HashMap::new(),
        }
    }

    pub fn global_env() -> Rc<RefCell<Self>> {
        let mut env = Self::new(None);
        env.init();
        Rc::new(RefCell::new(env))
    }

    pub fn insert(&mut self, name: &str, value: Object) {
        self.table.insert(name.to_string(), value);
    }

    pub fn set(&mut self, name: &str, value: Object) -> bool {
        if let Some(v) = self.table.get_mut(name) {
            *v = Rc::clone(&value);
            return true;
        }
        match &self.parent {
            None => false,
            Some(parent) => parent.borrow_mut().set(name, value),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        if let Some(v) = self.table.get(name) {
            return Some(Rc::clone(v));
        }
        match &self.parent {
            None => None,
            Some(parent) => parent.borrow().get(name),
        }
    }
}
