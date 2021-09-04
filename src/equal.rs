use super::object::{Cons, Object, ObjectKind};
use std::rc::Rc;

fn equal_cons(x: &Cons, y: &Cons) -> bool {
    if !equal(Rc::clone(&x.car), Rc::clone(&y.car)) {
        return false;
    }
    equal(Rc::clone(&x.cdr), Rc::clone(&y.cdr))
}

pub fn equal(x: Object, y: Object) -> bool {
    use ObjectKind::*;
    match &*x {
        Nil => match &*y {
            Nil => true,
            _ => false,
        },
        Fixnum(x) => match &*y {
            Fixnum(y) => x == y,
            _ => false,
        },
        Symbol(x) => match &*y {
            Symbol(y) => x == y,
            _ => false,
        },
        Cons(x) => match &*y {
            Cons(y) => equal_cons(x, y),
            _ => false,
        },
        Func(x) => match &*y {
            Func(y) => std::ptr::eq(x, y),
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::object::{cons, fixnum, nil, symbol};
    use super::*;
    #[test]
    fn nil_test() {
        assert!(equal(nil(), nil()));
        assert!(!equal(nil(), fixnum(1)));
    }

    #[test]
    fn fixnum_test() {
        assert!(equal(fixnum(1), fixnum(1)));
        assert!(!equal(fixnum(1), fixnum(2)));
        assert!(!equal(fixnum(1), nil()));
    }

    #[test]
    fn symbol_test() {
        assert!(equal(symbol("foo"), symbol("foo")));
        assert!(!equal(symbol("foo"), symbol("bar")));
        assert!(!equal(symbol("foo"), nil()));
        assert!(!equal(symbol("foo"), fixnum(1)));
    }

    #[test]
    fn cons_test() {
        assert!(equal(
            cons(fixnum(1), fixnum(2)),
            cons(fixnum(1), fixnum(2)),
        ));
        assert!(!equal(
            cons(fixnum(1), fixnum(2)),
            cons(fixnum(1), fixnum(3)),
        ));
        assert!(!equal(
            cons(fixnum(1), fixnum(2)),
            cons(fixnum(3), fixnum(2)),
        ));

        assert!(!equal(
            cons(symbol("+"), nil()),
            fixnum(100)
        ));
    }
}
