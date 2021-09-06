use core::fmt;

use super::object::{Object, ObjectType};

#[derive(Debug)]
pub enum RuntimeError {
    UnboundVariable(String),
    MismatchType(Object, ObjectType),
    WrongNumArgs(usize, usize),
    TooFewArguments(usize, usize),
    TooManyArguments(usize, usize),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::RuntimeError::*;
        match self {
            UnboundVariable(name) => write!(f, "Unbound variable: {}", name),
            MismatchType(value, expected_type) => {
                write!(f, "The value {} is not of type {:?}", value, expected_type)
            }
            WrongNumArgs(actual, expected) => write!(
                f,
                "Wrong number of arguments: expected = {}, actual = {}",
                expected, actual
            ),
            TooFewArguments(actual, min) => write!(
                f,
                "Too few arguments ({} arguments provided, at least {} required)",
                actual, min
            ),
            TooManyArguments(actual, max) => write!(
                f,
                "Too many arguments ({} arguments provided, at most {} required)",
                actual, max
            ),
        }
    }
}
