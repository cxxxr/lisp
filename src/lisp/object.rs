use core::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum ObjectKind {
    Cons(Cons),
    Fixnum(isize),
    Symbol(String),
    Nil,
}

pub type Object = Rc<ObjectKind>;

#[derive(Debug)]
pub struct Cons {
    car: Object,
    cdr: Object,
}

impl Cons {
    pub fn new(car: Object, cdr: Object) -> Cons {
        Cons { car, cdr }
    }
}

// constructor
pub fn cons(car: Object, cdr: Object) -> Object {
    Rc::new(ObjectKind::Cons(Cons::new(Rc::clone(&car), Rc::clone(&cdr))))
}

pub fn fixnum(n: isize) -> Object {
    Rc::new(ObjectKind::Fixnum(n))
}

pub fn symbol(s: &str) -> Object {
    Rc::new(ObjectKind::Symbol(s.to_string()))
}

pub fn nil() -> Object {
    Rc::new(ObjectKind::Nil)
}

// printer
impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectKind::Cons(cons) => cons.fmt(f),
            ObjectKind::Fixnum(n) => n.fmt(f),
            ObjectKind::Symbol(s) => s.fmt(f),
            ObjectKind::Nil => write!(f, "nil"),
        }
    }
}

impl fmt::Display for Cons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut cur = self;
        write!(f, "(")?;
        loop {
            write!(f, "{}", cur.car)?;
            match &*cur.cdr {
                ObjectKind::Cons(ref cons) => {
                    write!(f, " ")?;
                    cur = cons;
                    ()
                }
                ObjectKind::Nil => break write!(f, ")"),
                cdr => break write!(f, " . {})", cdr),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_fixnum_test() {
        let s = format!("{}", fixnum(123));
        assert_eq!(s, "123");
    }

    #[test]
    fn display_symbol_test() {
        let s = format!("{}", symbol("car"));
        assert_eq!(s, "car");
    }

    #[test]
    fn display_cons_test() {
        let obj = cons(fixnum(1), fixnum(2));
        let s = format!("{}", obj);
        assert_eq!(s, "(1 . 2)");

        let obj = cons(fixnum(1), cons(fixnum(2), fixnum(3)));
        let s = format!("{}", obj);
        assert_eq!(s, "(1 2 . 3)");

        let obj = cons(cons(fixnum(1), fixnum(2)), cons(fixnum(3), fixnum(4)));
        let s = format!("{}", obj);
        assert_eq!(s, "((1 . 2) 3 . 4)");

        let obj = cons(symbol("+"), cons(fixnum(123), cons(fixnum(456), nil())));
        let s = format!("{}", obj);
        assert_eq!(s, "(+ 123 456)");
    }
}
