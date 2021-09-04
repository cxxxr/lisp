use std::collections::HashMap;
use std::rc::Rc;

use super::equal;
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

fn check_num_args_range(args: &[Object], min: usize, max: usize) -> Result<(), RuntimeError> {
    if args.len() < min {
        return Err(RuntimeError::TooFewArguments(args.len(), min));
    }
    if max < args.len() {
        return Err(RuntimeError::TooManyArguments(args.len(), max));
    }
    Ok(())
}

fn eval_quote(args: &[Object]) -> Result<Object, RuntimeError> {
    check_num_args(&args, 1)?;
    return Ok(Rc::clone(&args[0]));
}

fn eval_if(args: &[Object], env: &mut Env) -> Result<Object, RuntimeError> {
    check_num_args_range(&args, 2, 3)?;
    match &*eval_internal(Rc::clone(&args[0]), env)? {
        ObjectKind::Nil => match args.get(2) {
            Some(x) => eval_internal(Rc::clone(x), env),
            None => Ok(object::nil()),
        },
        _ => eval_internal(Rc::clone(&args[1]), env),
    }
}

fn eval_function(
    first: Object,
    iter: object::ListIter,
    env: &mut Env,
) -> Result<Object, RuntimeError> {
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
                        return eval_quote(&args);
                    }
                    "if" => {
                        let args: Vec<Object> = iter.collect();
                        return eval_if(&args, env);
                    }
                    _ => (),
                }
            }
            eval_function(first, iter, env)
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

fn is_atom(args: &[Object]) -> Result<Object, RuntimeError> {
    check_num_args(args, 1)?;
    match &*args[0] {
        ObjectKind::Cons(_) => Ok(object::nil()),
        _ => Ok(object::symbol("t")),
    }
}

fn cons(args: &[Object]) -> Result<Object, RuntimeError> {
    check_num_args(args, 2)?;
    Ok(object::cons(Rc::clone(&args[0]), Rc::clone(&args[1])))
}

fn cxr<F>(args: &[Object], accessor: F) -> Result<Object, RuntimeError>
where
    F: Fn(&object::Cons) -> Object,
{
    check_num_args(args, 1)?;
    match &*args[0] {
        ObjectKind::Cons(cons) => Ok(accessor(cons)),
        _ => Err(RuntimeError::MismatchType(
            Rc::clone(&args[0]),
            ObjectType::Cons,
        )),
    }
}

fn car(args: &[Object]) -> Result<Object, RuntimeError> {
    cxr(args, |cons| Rc::clone(&cons.car))
}

fn cdr(args: &[Object]) -> Result<Object, RuntimeError> {
    cxr(args, |cons| Rc::clone(&cons.cdr))
}

fn equal(args: &[Object]) -> Result<Object, RuntimeError> {
    check_num_args(args, 2)?;
    if equal::equal(Rc::clone(&args[0]), Rc::clone(&args[1])) {
        Ok(object::symbol("t"))
    } else {
        Ok(object::nil())
    }
}

fn global_env() -> Env {
    let mut genv = Env::new();
    genv.set("+", Object::new(ObjectKind::Func(plus)));
    genv.set("atom?", Object::new(ObjectKind::Func(is_atom)));
    genv.set("cons", Object::new(ObjectKind::Func(cons)));
    genv.set("car", Object::new(ObjectKind::Func(car)));
    genv.set("cdr", Object::new(ObjectKind::Func(cdr)));
    genv.set("equal", Object::new(ObjectKind::Func(equal)));
    genv
}

pub fn eval(x: Object) -> Result<Object, RuntimeError> {
    let mut e = global_env();
    eval_internal(x, &mut e)
}
