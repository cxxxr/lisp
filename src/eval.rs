use std::cell::RefCell;
use std::rc::Rc;

use super::env::Env;
use super::equal;
use super::object::{self, Object, ObjectKind, ObjectType};
use super::error::RuntimeError;

pub type EvalResult = Result<Object, RuntimeError>;

fn check_num_args(args: &[Object], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::WrongNumArgs(args.len(), expected));
    }
    Ok(())
}

fn check_num_args_range(
    args: &[Object],
    min: usize,
    max: impl Into<Option<usize>>,
) -> Result<(), RuntimeError> {
    if args.len() < min {
        return Err(RuntimeError::TooFewArguments(args.len(), min));
    }
    if let Some(max) = max.into() {
        if max < args.len() {
            return Err(RuntimeError::TooManyArguments(args.len(), max));
        }
    }
    Ok(())
}

fn eval_quote(args: &[Object]) -> EvalResult {
    check_num_args(&args, 1)?;
    return Ok(Rc::clone(&args[0]));
}

fn eval_if(args: &[Object], env: Rc<RefCell<Env>>) -> EvalResult {
    check_num_args_range(&args, 2, 3)?;
    match &*eval_internal(Rc::clone(&args[0]), Rc::clone(&env))? {
        ObjectKind::Nil => match args.get(2) {
            Some(x) => eval_internal(Rc::clone(x), Rc::clone(&env)),
            None => Ok(object::nil()),
        },
        _ => eval_internal(Rc::clone(&args[1]), Rc::clone(&env)),
    }
}

fn eval_define(args: &[Object], env: Rc<RefCell<Env>>) -> EvalResult {
    check_num_args(&args, 2)?;
    let var = Rc::clone(&args[0]);
    let value = Rc::clone(&args[1]);

    let name = match &*var {
        ObjectKind::Symbol(name) => name,
        _ => return Err(RuntimeError::MismatchType(var, ObjectType::Symbol)),
    };

    let value = eval_internal(value, Rc::clone(&env))?;
    env.borrow_mut().insert(name, Rc::clone(&value));
    Ok(value)
}

fn eval_lambda(args_iter: &mut object::ListIter, env: Rc<RefCell<Env>>) -> EvalResult {
    let list = match args_iter.next() {
        Some(arg) => match &*arg {
            ObjectKind::Cons(cons) => cons.iter().collect(),
            ObjectKind::Nil => Vec::new(),
            _ => return Err(RuntimeError::MismatchType(arg, ObjectType::List)),
        },
        None => return Err(RuntimeError::TooFewArguments(0, 1)),
    };

    let mut params = Vec::new();
    for param in list {
        match &*param {
            ObjectKind::Symbol(name) => params.push(name.clone()), // XXX
            _ => {
                return Err(RuntimeError::MismatchType(
                    Rc::clone(&param),
                    ObjectType::Symbol,
                ))
            }
        }
    }

    return Ok(object::closure(params, args_iter.collect(), env));
}

fn eval_set(args: &[Object], env: Rc<RefCell<Env>>) -> EvalResult {
    check_num_args(&args, 2)?;
    let var = Rc::clone(&args[0]);
    let value = Rc::clone(&args[1]);

    let name = match &*var {
        ObjectKind::Symbol(name) => name,
        _ => return Err(RuntimeError::MismatchType(var, ObjectType::Symbol)),
    };

    let value = eval_internal(value, Rc::clone(&env))?;
    let mut env = env.borrow_mut();
    if env.set(name, Rc::clone(&value)) {
        Ok(value)
    } else {
        Err(RuntimeError::UnboundVariable(name.clone()))
    }
}

fn apply_closure(closure: &object::Closure, args: Vec<Object>) -> EvalResult {
    if closure.parameters.len() != args.len() {
        return Err(RuntimeError::WrongNumArgs(
            args.len(),
            closure.parameters.len(),
        ));
    }

    let parent = Rc::clone(&closure.env);
    let env = Rc::new(RefCell::new(Env::new(Some(parent))));

    for (param, arg) in closure.parameters.iter().zip(args.iter()) {
        env.borrow_mut().insert(param, Rc::clone(arg));
    }

    let mut result = object::nil();
    for form in closure.body.iter() {
        result = eval_internal(Rc::clone(form), Rc::clone(&env))?;
    }

    Ok(result)
}

fn apply_function(first: Object, iter: object::ListIter, env: Rc<RefCell<Env>>) -> EvalResult {
    let first = eval_internal(first, Rc::clone(&env))?;
    let mut args = Vec::new();
    for arg in iter {
        args.push(eval_internal(arg, Rc::clone(&env))?);
    }

    match &*first {
        ObjectKind::Func(func) => func(&args),
        ObjectKind::Closure(closure) => apply_closure(closure, args),
        _ => Err(RuntimeError::MismatchType(first, ObjectType::Function)),
    }
}

fn eval_internal(x: Object, env: Rc<RefCell<Env>>) -> EvalResult {
    match &*x {
        ObjectKind::Nil | ObjectKind::Fixnum(_) | ObjectKind::Func(_) | ObjectKind::Closure(_) => {
            Ok(x)
        }
        ObjectKind::Symbol(s) => env
            .borrow()
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
                    "define" => {
                        let args: Vec<Object> = iter.collect();
                        return eval_define(&args, env);
                    }
                    "lambda" => {
                        return eval_lambda(&mut iter, env);
                    }
                    "set!" => {
                        let args: Vec<Object> = iter.collect();
                        return eval_set(&args, env);
                    }
                    _ => (),
                }
            }
            apply_function(first, iter, env)
        }
    }
}

mod builtin {
    use super::object::Object;
    use super::*;

    pub fn plus(args: &[Object]) -> EvalResult {
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

    pub fn is_atom(args: &[Object]) -> EvalResult {
        check_num_args(args, 1)?;
        match &*args[0] {
            ObjectKind::Cons(_) => Ok(object::nil()),
            _ => Ok(object::symbol("t")),
        }
    }

    pub fn cons(args: &[Object]) -> EvalResult {
        check_num_args(args, 2)?;
        Ok(object::cons(Rc::clone(&args[0]), Rc::clone(&args[1])))
    }

    fn cxr<F>(args: &[Object], accessor: F) -> EvalResult
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

    pub fn car(args: &[Object]) -> EvalResult {
        cxr(args, |cons| Rc::clone(&cons.car))
    }

    pub fn cdr(args: &[Object]) -> EvalResult {
        cxr(args, |cons| Rc::clone(&cons.cdr))
    }

    pub fn equal(args: &[Object]) -> EvalResult {
        check_num_args(args, 2)?;
        if equal::equal(Rc::clone(&args[0]), Rc::clone(&args[1])) {
            Ok(object::symbol("t"))
        } else {
            Ok(object::nil())
        }
    }
}

impl Env {
    pub fn init(&mut self) {
        self.insert("+", Object::new(ObjectKind::Func(builtin::plus)));
        self.insert("atom?", Object::new(ObjectKind::Func(builtin::is_atom)));
        self.insert("cons", Object::new(ObjectKind::Func(builtin::cons)));
        self.insert("car", Object::new(ObjectKind::Func(builtin::car)));
        self.insert("cdr", Object::new(ObjectKind::Func(builtin::cdr)));
        self.insert("equal", Object::new(ObjectKind::Func(builtin::equal)));
    }
}

pub fn eval(x: Object, env: Rc<RefCell<Env>>) -> EvalResult {
    eval_internal(x, env)
}
