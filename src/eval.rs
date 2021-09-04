use std::collections::HashMap;
use std::rc::Rc;

use super::object::{self, Cons, Error, Object, ObjectKind};

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
            let mut iter = list.iter();
            let first = iter.next().unwrap();
            let first = eval_internal(first, env)?;
            let func = match &*first {
                ObjectKind::Func(func) => func,
                _ => return Err(Error::MismatchType(first)),
            };
            let mut args = Vec::new();
            for arg in iter {
                args.push(eval_internal(arg, env)?);
            }
            func(&args)
        }
    }
}

fn plus(args: &[Object]) -> Result<Object, Error> {
    let mut acc = 0;
    for arg in args {
        match **arg {
            ObjectKind::Fixnum(n) => {
                acc += n;
            }
            _ => (),
        }
    }
    Ok(object::fixnum(acc))
}

fn global_env() -> Env {
    let mut genv = Env::new();
    genv.set("+", Object::new(ObjectKind::Func(plus)));
    genv
}

pub fn eval(x: Object) -> Result<Object, Error> {
    let mut e = global_env();
    eval_internal(x, &mut e)
}
