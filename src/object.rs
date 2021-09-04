use core::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum ObjectType {
    Number,
    Function,
}

#[derive(Debug)]
pub enum RuntimeError {
    UnboundVariable(String),
    MismatchType(MismatchTypeError),
}

#[derive(Debug)]
pub struct MismatchTypeError {
    value: Object,
    expected_type: ObjectType,
}

impl MismatchTypeError {
    pub fn new(value: Object, expected_type: ObjectType) -> Self {
        Self {
            value,
            expected_type,
        }
    }
}

pub type Object = Rc<ObjectKind>;
pub enum ObjectKind {
    Nil,
    Fixnum(isize),
    Symbol(String),
    Cons(Cons),
    Func(fn(&[Object]) -> Result<Object, RuntimeError>),
}

#[derive(Debug)]
pub struct Cons {
    pub car: Object,
    pub cdr: Object,
}

impl Cons {
    pub fn new(car: Object, cdr: Object) -> Self {
        Self { car, cdr }
    }

    pub fn iter(&self) -> ListIter {
        ListIter {
            cons: self,
            is_end: false,
        }
    }
}

pub struct ListIter<'a> {
    cons: &'a Cons,
    is_end: bool,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end {
            return None;
        }
        match &*self.cons.cdr {
            ObjectKind::Cons(cons) => {
                let result = Some(Rc::clone(&self.cons.car));
                self.cons = cons;
                result
            }
            _ => {
                self.is_end = true;
                Some(Rc::clone(&self.cons.car))
            }
        }
    }
}

pub fn cons(car: Object, cdr: Object) -> Object {
    Rc::new(ObjectKind::Cons(Cons::new(
        Rc::clone(&car),
        Rc::clone(&cdr),
    )))
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

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectKind::Nil => write!(f, "nil"),
            ObjectKind::Fixnum(n) => n.fmt(f),
            ObjectKind::Symbol(s) => s.fmt(f),
            ObjectKind::Cons(cons) => cons.fmt(f),
            ObjectKind::Func(func) => write!(f, "<Fn {:p}>", &func),
        }
    }
}

impl fmt::Debug for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::RuntimeError::*;
        match self {
            UnboundVariable(name) => write!(f, "Unbound variable: {}", name),
            MismatchType(e) => write!(
                f,
                "The value {} is not of type {:?}",
                e.value, e.expected_type
            ),
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
