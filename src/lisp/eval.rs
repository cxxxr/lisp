use std::collections::HashMap;
use std::rc::Rc;

use super::object::{Object, ObjectKind, Error, Cons};

struct Env {
    parent: Option<Box<Env>>,
    table: HashMap<String, Object>,
}

impl Env {
    fn new() -> Self {
        Self {
            parent: None,
            table: HashMap::new(),
        }
    }

    fn set(&mut self, name: &str, value: Object) {
        self.table.insert(name.to_string(), value);
    }

    fn get(&self, name: &str) -> Option<Object> {
        match self.table.get(name) {
            None => None,
            Some(v) => Some(Rc::clone(v)),
        }
    }
}

fn eval_internal(x: Object, env: &mut Env) -> Result<Object, Error> {
    match &*x {
        ObjectKind::Nil | ObjectKind::Fixnum(_) | ObjectKind::Func(_) => Ok(x),
        ObjectKind::Symbol(s) => env.get(s).ok_or(Error::UnboundVariable(s.to_string())),
        ObjectKind::Cons(list) => {
            unimplemented!()
        }
    }
}

pub fn eval(x: Object) -> Result<Object, Error> {
    let mut e = Env::new();
    eval_internal(x, &mut e)
}
