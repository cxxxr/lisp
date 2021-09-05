use lisp::{
    env::Env,
    equal::equal,
    eval::{eval, EvalResult},
    object::{cons, fixnum, nil, symbol, Object, ObjectType, RuntimeError},
    reader::read_from_string,
};

use std::cell::RefCell;
use std::rc::Rc;

extern crate lisp;

fn call_eval_with_env(input: &str, env: Rc<RefCell<Env>>) -> EvalResult {
    let x = match read_from_string(input) {
        Ok((x, _)) => x,
        _ => unreachable!(),
    };
    eval(x, env)
}

fn call_eval(input: &str) -> EvalResult {
    call_eval_with_env(input, Env::global_env())
}

fn assert_eval(expected: Object, result: EvalResult) {
    let actual = result.unwrap();
    assert!(equal(actual, expected));
}

fn verify_eval(expected: Object, input: &str) {
    assert_eval(expected, call_eval(input));
}

fn verify_eval_with_env(expected: Object, input: &str, env: Rc<RefCell<Env>>) {
    assert_eval(expected, call_eval_with_env(input, env));
}

#[test]
fn atom_test() {
    verify_eval(symbol("t"), "(atom? 1)");
    verify_eval(symbol("t"), "(atom? 'foo)");
    verify_eval(symbol("t"), "(atom? 'foo)");
    verify_eval(nil(), "(atom? (cons 1 2))");
    assert!(match call_eval("(atom?)") {
        Err(RuntimeError::WrongNumArgs(0, 1)) => true,
        _ => false,
    });
}

#[test]
fn add_test() {
    verify_eval(fixnum(0), "(+)");
    verify_eval(fixnum(1), "(+ 1)");
    verify_eval(fixnum(3), "(+ 1 2)");
    verify_eval(fixnum(6), "(+ 1 2 3)");
    assert!(match call_eval("(+ 'a)") {
        Err(RuntimeError::MismatchType(_, ObjectType::Number)) => true,
        _ => false,
    });
}

#[test]
fn quote_test() {
    verify_eval(symbol("a"), "'a");
    assert!(match call_eval("(quote)") {
        Err(RuntimeError::WrongNumArgs(0, 1)) => true,
        _ => false,
    });
}

#[test]
fn cons_test() {
    verify_eval(cons(symbol("a"), symbol("b")), "(cons 'a 'b)");
    verify_eval(
        cons(cons(fixnum(1), fixnum(2)), cons(fixnum(3), fixnum(4))),
        "(cons (cons 1 2) (cons 3 4))",
    );
    assert!(match call_eval("(cons)") {
        Err(RuntimeError::WrongNumArgs(0, 2)) => true,
        _ => false,
    });
}

#[test]
fn car_test() {
    verify_eval(fixnum(1), "(car (cons 1 2))");
    assert!(match call_eval("(car)") {
        Err(RuntimeError::WrongNumArgs(0, 1)) => true,
        _ => false,
    });
    assert!(match call_eval("(car 'a)") {
        Err(RuntimeError::MismatchType(_, ObjectType::Cons)) => true,
        _ => false,
    });
}

#[test]
fn cdr_test() {
    verify_eval(fixnum(2), "(cdr (cons 1 2))");
    assert!(match call_eval("(cdr)") {
        Err(RuntimeError::WrongNumArgs(0, 1)) => true,
        _ => false,
    });
    assert!(match call_eval("(cdr 'a)") {
        Err(RuntimeError::MismatchType(_, ObjectType::Cons)) => true,
        _ => false,
    });
}

#[test]
fn if_test() {
    verify_eval(symbol("true"), "(if (equal 1 1) 'true 'false)");
    verify_eval(symbol("false"), "(if (equal 1 2) 'true 'false)");
    verify_eval(nil(), "(if (equal 1 2) 'true)");
    assert!(match call_eval("(if)") {
        Err(RuntimeError::TooFewArguments(0, 2)) => true,
        _ => false,
    });
    assert!(match call_eval("(if x)") {
        Err(RuntimeError::TooFewArguments(1, 2)) => true,
        _ => false,
    });
    assert!(match call_eval("(if test then else extra)") {
        Err(RuntimeError::TooManyArguments(4, 3)) => true,
        _ => false,
    });
}

#[test]
fn define_test() {
    let env = Env::global_env();
    verify_eval_with_env(fixnum(1), "(define x 1)", Rc::clone(&env));
    verify_eval_with_env(fixnum(2), "(define x (+ x 1))", env);
}

#[test]
fn lambda_test() {
    let env = Env::global_env();
    assert!(call_eval_with_env("(define 1+ (lambda (x) (+ x 1)))", Rc::clone(&env)).is_ok());
    verify_eval_with_env(fixnum(1), "(1+ 0)", env);
}

#[test]
fn set_test() -> Result<(), RuntimeError> {
    assert!(match call_eval("(set! x 0)") {
        Err(RuntimeError::UnboundVariable(var)) if var == "x" => true,
        _ => false,
    });
    let env = Env::global_env();
    call_eval_with_env("(define foo nil)", Rc::clone(&env))?;
    verify_eval_with_env(fixnum(10), "(set! foo 10)", Rc::clone(&env));
    verify_eval_with_env(fixnum(10), "foo", Rc::clone(&env));
    Ok(())
}

#[test]
fn closure_test() -> Result<(), RuntimeError> {
    let env = Env::global_env();
    call_eval_with_env("(define mkcounter (lambda () (define counter 0) (lambda () (set! counter (+ counter 1)) counter)))", Rc::clone(&env))?;
    call_eval_with_env("(define c (mkcounter))", Rc::clone(&env))?;
    verify_eval_with_env(fixnum(1), "(c)", Rc::clone(&env));
    verify_eval_with_env(fixnum(2), "(c)", Rc::clone(&env));
    verify_eval_with_env(fixnum(3), "(c)", Rc::clone(&env));
    Ok(())
}
