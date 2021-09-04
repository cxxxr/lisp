use lisp::{
    equal::equal,
    eval::eval,
    object::{fixnum, Object},
    reader::read_from_string,
};

extern crate lisp;

fn verify_eval(input: &str, expected: Object) {
    let x = match read_from_string(input) {
        Ok((x, _)) => x,
        _ => unreachable!(),
    };
    let actual = match eval(x) {
        Ok(actual) => actual,
        _ => unreachable!(),
    };
    assert!(equal(actual, expected));
}

#[test]
fn read_eval() {
    verify_eval("(+)", fixnum(0));
    verify_eval("(+ 1)", fixnum(1));
    verify_eval("(+ 1 2)", fixnum(3));
}
