use std::collections::HashMap;
use std::rc::Rc;

use super::object::{self, Object, ObjectKind, ObjectType, RuntimeError};

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

fn check_num_args(args: &[Object], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongNumArgs(args.len(), expected));
    }
    Ok(())
}

fn eval_internal(x: Object, env: &mut Env) -> Result<Object, RuntimeError> {
    match &*x {
        ObjectKind::Nil | ObjectKind::Fixnum(_) | ObjectKind::Func(_) => Ok(x),
        ObjectKind::Symbol(s) => env
            .get(s)
            .ok_or(RuntimeError::UnboundVariable(s.to_string())),
        ObjectKind::Cons(list) => {
            let mut iter = list.iter();
            let first = iter.next().unwrap();

            if let ObjectKind::Symbol(name) = &*first {
                match &**name {
                    "quote" => {
                        let args: Vec<Object> = iter.collect();
                        check_num_args(&args, 1)?;
                        return Ok(Rc::clone(&args[0]));
                    }
                    _ => (),
                }
            }

            let first = eval_internal(first, env)?;
            let func = match &*first {
                ObjectKind::Func(func) => func,
                _ => return Err(RuntimeError::MismatchType(first, ObjectType::Function)),
            };
            let mut args = Vec::new();
            for arg in iter {
                args.push(eval_internal(arg, env)?);
            }
            func(&args)
        }
    }
}

fn plus(args: &[Object]) -> Result<Object, RuntimeError> {
    let mut acc = 0;
    for arg in args {
        match **arg {
            ObjectKind::Fixnum(n) => {
                acc += n;
            }
            _ => {
                return Err(RuntimeError::MismatchType(
                    Rc::clone(arg),
                    ObjectType::Number,
                ))
            }
        }
    }
    Ok(object::fixnum(acc))
}

fn cons(args: &[Object]) -> Result<Object, RuntimeError> {
    check_num_args(args, 2)?;
    Ok(object::cons(Rc::clone(&args[0]), Rc::clone(&args[1])))
}

fn car(args: &[Object]) -> Result<Object, RuntimeError> {
    check_num_args(args, 1)?;
    match &*args[0] {
        ObjectKind::Cons(cons) => Ok(Rc::clone(&cons.car)),
        _ => Err(RuntimeError::MismatchType(
            Rc::clone(&args[0]),
            ObjectType::Cons,
        )),
    }
}

fn global_env() -> Env {
    let mut genv = Env::new();
    genv.set("+", Object::new(ObjectKind::Func(plus)));
    genv.set("cons", Object::new(ObjectKind::Func(cons)));
    genv.set("car", Object::new(ObjectKind::Func(car)));
    genv
}

pub fn eval(x: Object) -> Result<Object, RuntimeError> {
    let mut e = global_env();
    eval_internal(x, &mut e)
}
