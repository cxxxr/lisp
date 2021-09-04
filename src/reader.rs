use super::object;
use core::fmt;
use std::str::from_utf8;

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

type ReadResult = Result<(object::Object, usize), ReadError>;

fn skip_spaces(input: &[u8], mut pos: usize) -> usize {
    while pos < input.len() && input[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

fn peek_char(input: &[u8], pos: usize) -> Result<u8, ReadError> {
    if pos < input.len() {
        Ok(input[pos])
    } else {
        Err(ReadError::EndOfFile)
    }
}

fn vec_to_cons(mut vec: Vec<object::Object>, last: object::Object) -> object::Object {
    let mut head = last;
    vec.reverse();
    for x in vec {
        head = object::cons(x, head);
    }
    head
}

fn read_list(input: &[u8], mut pos: usize) -> ReadResult {
    let mut list = Vec::<object::Object>::new();

    pos = skip_spaces(input, pos);
    match peek_char(input, pos)? {
        b')' => return Ok((object::nil(), pos)),
        _ => (),
    }

    let last = loop {
        let (obj, end) = read_ahead(input, pos)?;
        pos = skip_spaces(input, end);
        list.push(obj);
        match peek_char(input, pos)? {
            b'.' => {
                let (last, pos) = read_ahead(input, pos + 1)?;
                let pos = skip_spaces(input, pos);
                match peek_char(input, pos)? {
                    b')' => break last,
                    b => {
                        return Err(ReadError::UnexpectedChar(
                            b as char, // TODO: multibyte char
                            ')',
                        ));
                    }
                }
            }
            b')' => break object::nil(),
            _ => (),
        }
    };
    Ok((vec_to_cons(list, last), pos + 1))
}

fn is_delimiter(b: u8) -> bool {
    match b {
        b'(' | b')' | b'\'' => true,
        b if b.is_ascii_whitespace() => true,
        _ => false,
    }
}

fn read_atom(input: &[u8], mut pos: usize) -> ReadResult {
    let start = pos;
    while pos < input.len() && !is_delimiter(input[pos]) {
        pos += 1;
    }
    let s = from_utf8(&input[start..pos]).unwrap();
    let obj = match s.parse() {
        Ok(n) => object::fixnum(n),
        _ => object::symbol(&s),
    };
    Ok((obj, pos))
}

fn read_quote(input: &[u8], pos: usize) -> ReadResult {
    let (obj, pos) = read_ahead(input, pos)?;
    let obj = object::cons(object::symbol("quote"), object::cons(obj, object::nil()));
    Ok((obj, pos))
}

fn read_ahead(input: &[u8], mut pos: usize) -> ReadResult {
    pos = skip_spaces(input, pos);

    if pos >= input.len() {
        return Err(ReadError::EndOfFile);
    }

    match input[pos] {
        b'(' => read_list(input, pos + 1),
        b')' => return Err(ReadError::UnmatchedClosedParen),
        b'\'' => read_quote(input, pos + 1),
        _ => read_atom(input, pos),
    }
}

pub fn read_from_string(input: &str) -> ReadResult {
    let input = input.as_bytes();
    read_ahead(input, 0)
}

#[cfg(test)]
mod tests {
    use super::super::equal::equal;
    use super::super::object::*;
    use super::read_from_string;

    fn verify(input: &str, expected: Object) {
        assert!(match read_from_string(input) {
            Ok((x, _)) => equal(x, expected),
            _ => false,
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
    }
}
