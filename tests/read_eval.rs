use lisp::{
    equal::equal,
    eval::eval,
    object::{cons, fixnum, nil, symbol, Object, ObjectType, RuntimeError},
    reader::read_from_string,
};

extern crate lisp;

fn call_eval(input: &str) -> Result<Object, RuntimeError> {
    let x = match read_from_string(input) {
        Ok((x, _)) => x,
        _ => unreachable!(),
    };
    eval(x)
}

fn verify_eval(input: &str, expected: Object) {
    let actual = call_eval(input).unwrap();
    assert!(equal(actual, expected));
}

#[test]
fn atom_test() {
    verify_eval("(atom? 1)", symbol("t"));
    verify_eval("(atom? 'foo)", symbol("t"));
    verify_eval("(atom? 'foo)", symbol("t"));
    verify_eval("(atom? (cons 1 2))", nil());
    assert!(match call_eval("(atom?)") {
        Err(RuntimeError::WrongNumArgs(0, 1)) => true,
        _ => false,
    });
}

#[test]
fn add_test() {
    verify_eval("(+)", fixnum(0));
    verify_eval("(+ 1)", fixnum(1));
    verify_eval("(+ 1 2)", fixnum(3));
    verify_eval("(+ 1 2 3)", fixnum(6));
    assert!(match call_eval("(+ 'a)") {
        Err(RuntimeError::MismatchType(_, ObjectType::Number)) => true,
        _ => false,
    });
}

#[test]
fn quote_test() {
    verify_eval("'a", symbol("a"));
    assert!(match call_eval("(quote)") {
        Err(RuntimeError::WrongNumArgs(0, 1)) => true,
        _ => false,
    });
}

#[test]
fn cons_test() {
    verify_eval("(cons 'a 'b)", cons(symbol("a"), symbol("b")));
    verify_eval(
        "(cons (cons 1 2) (cons 3 4))",
        cons(cons(fixnum(1), fixnum(2)), cons(fixnum(3), fixnum(4))),
    );
    assert!(match call_eval("(cons)") {
        Err(RuntimeError::WrongNumArgs(0, 2)) => true,
        _ => false,
    });
}

#[test]
fn car_test() {
    verify_eval("(car (cons 1 2))", fixnum(1));
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
    verify_eval("(cdr (cons 1 2))", fixnum(2));
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
    verify_eval("(if (equal 1 1) 'true 'false)", symbol("true"));
    verify_eval("(if (equal 1 2) 'true 'false)", symbol("false"));
    verify_eval("(if (equal 1 2) 'true)", nil());
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
