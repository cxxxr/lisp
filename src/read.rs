use crate::reader::Reader;

use super::object;
use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReadError {
    EndOfFile,
    UnmatchedClosedParen,
    UnexpectedChar(char, char),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ReadError::*;
        match self {
            EndOfFile => write!(f, "End of file"),
            UnmatchedClosedParen => write!(f, "Unmatched closed parenthesis"),
            UnexpectedChar(actual, expected) => write!(
                f,
                "Expecting character {:?}, but it's character {:?}",
                expected, actual
            ),
        }
    }
}

pub fn read_from_string(input: &str) -> Result<(object::Object, usize), ReadError> {
    let mut r = super::reader::StringStream::new(input);
    r.read_ahead().map(|x| (x, r.pos()))
}

#[cfg(test)]
mod tests {
    use super::super::equal::equal;
    use super::super::object::*;
    use super::read_from_string;

    fn verify(input: &str, expected: Object) {
        assert!(match read_from_string(input) {
            Ok((x, _)) => {
                println!("!!!result: {}", x);
                equal(x, expected)
            }
            _ => {
                println!("error: {}", input);
                false
            }
        });
    }

    #[test]
    fn test() {
        verify("a", symbol("a"));
        verify("  a", symbol("a"));
        verify("123", fixnum(123));
        verify("-123", fixnum(-123));
        verify("+123", fixnum(123));
        verify("()", nil());
        verify("nil", nil());
        verify("(+)", cons(symbol("+"), nil()));
        verify(
            "(a b c)",
            cons(symbol("a"), cons(symbol("b"), cons(symbol("c"), nil()))),
        );
        verify("(a . b)", cons(symbol("a"), symbol("b")));
        verify("'foo", cons(symbol("quote"), cons(symbol("foo"), nil())));
        verify(
            "'(a b)",
            cons(
                symbol("quote"),
                cons(cons(symbol("a"), cons(symbol("b"), nil())), nil()),
            ),
        );
        verify("(() 1)", cons(nil(), cons(fixnum(1), nil())));
    }
}
