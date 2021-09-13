use super::object;
use super::read::ReadError;
use std::io::{self, BufRead};
use std::str::from_utf8;

type ReadResult = Result<object::Object, ReadError>;

fn vec_to_cons(mut vec: Vec<object::Object>, last: object::Object) -> object::Object {
    let mut head = last;
    vec.reverse();
    for x in vec {
        head = object::cons(x, head);
    }
    head
}

fn is_delimiter(b: u8) -> bool {
    match b {
        b'(' | b')' | b'\'' => true,
        b if b.is_ascii_whitespace() => true,
        _ => false,
    }
}
pub trait Reader {
    fn peek_char(&self) -> Result<u8, ReadError>;
    fn next_char(&mut self) -> Result<u8, ReadError>;

    fn skip_spaces(&mut self) {
        loop {
            match self.peek_char() {
                Ok(c) if c.is_ascii_whitespace() => {
                    self.next_char().unwrap();
                }
                _ => return,
            }
        }
    }

    fn read_list(&mut self) -> ReadResult {
        let mut list = Vec::<object::Object>::new();

        self.skip_spaces();
        match self.peek_char()? {
            b')' => {
                self.next_char().unwrap();
                return Ok(object::nil())
            },
            _ => (),
        }

        let last = loop {
            let obj = self.read_ahead()?;
            self.skip_spaces();
            list.push(obj);
            match self.peek_char()? {
                b'.' => {
                    self.next_char().unwrap();
                    let last = self.read_ahead()?;
                    self.skip_spaces();
                    match self.peek_char()? {
                        b')' => break last,
                        b => {
                            return Err(ReadError::UnexpectedChar(
                                b as char, // TODO: multibyte char
                                ')',
                            ));
                        }
                    }
                }
                b')' => {
                    self.next_char().unwrap();
                    break object::nil()
                },
                _ => (),
            }
        };
        Ok(vec_to_cons(list, last))
    }

    fn read_atom(&mut self) -> ReadResult {
        let mut v = Vec::new();
        loop {
            match self.peek_char() {
                Ok(c) if is_delimiter(c) => break,
                Ok(c) => {
                    v.push(c);
                    self.next_char()?;
                }
                Err(_) => break,
            }
        }
        let s = from_utf8(&v).unwrap();
        let obj = match s.parse() {
            Ok(n) => object::fixnum(n),
            _ => object::symbol(&s),
        };
        Ok(obj)
    }

    fn read_quote(&mut self) -> ReadResult {
        let obj = self.read_ahead()?;
        let obj = object::cons(object::symbol("quote"), object::cons(obj, object::nil()));
        Ok(obj)
    }

    fn read_ahead(&mut self) -> ReadResult {
        self.skip_spaces();

        match self.peek_char()? {
            b')' => Err(ReadError::UnmatchedClosedParen),
            b'(' => {
                self.next_char().unwrap();
                self.read_list()
            },
            b'\'' => {
                self.next_char().unwrap();
                self.read_quote()
            },
            _ => self.read_atom(),
        }
    }
}

pub struct StringStream {
    buffer: Vec<u8>,
    pos: usize,
}

impl StringStream {
    pub fn new(str: &str) -> Self {
        Self {
            buffer: str.as_bytes().to_vec(),
            pos: 0,
        }
    }

    fn update(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
        self.pos = 0;
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl Reader for StringStream {
    fn peek_char(&self) -> Result<u8, ReadError> {
        if self.pos < self.buffer.len() {
            Ok(self.buffer[self.pos])
        } else {
            Err(ReadError::EndOfFile)
        }
    }

    fn next_char(&mut self) -> Result<u8, ReadError> {
        self.peek_char().and_then(|c| {
            self.pos += 1;
            Ok(c)
        })
    }
}

pub struct InputStream<R> {
    rdr: io::BufReader<R>,
    inner: StringStream,
}

impl<R: io::Read> InputStream<R> {
    pub fn from_reader(rdr: R) -> Self {
        InputStream {
            rdr: io::BufReader::new(rdr),
            inner: StringStream::new(""),
        }
    }

    fn read_line(&mut self) -> Option<()> {
        let mut buf = String::new();
        match self.rdr.read_line(&mut buf) {
            Ok(_) => {
                self.inner.update(buf.trim_end().as_bytes().to_vec());
                Some(())
            }
            Err(_) => None,
        }
    }
}

impl<R: io::Read> Reader for InputStream<R> {
    fn peek_char(&self) -> Result<u8, ReadError> {
        self.inner.peek_char()
    }

    fn next_char(&mut self) -> Result<u8, ReadError> {
        match self.inner.next_char() {
            Ok(c) => Ok(c),
            Err(ReadError::EndOfFile) => {
                self.read_line();
                self.inner.next_char()
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_stream() {
        let mut s = StringStream::new("abc");
        assert_eq!(s.next_char(), Ok(b'a'));
        assert_eq!(s.next_char(), Ok(b'b'));
        assert_eq!(s.next_char(), Ok(b'c'));
        assert_eq!(s.next_char(), Err(ReadError::EndOfFile));
        s.update("xyz".as_bytes().to_vec());
        assert_eq!(s.next_char(), Ok(b'x'));
        assert_eq!(s.next_char(), Ok(b'y'));
        assert_eq!(s.next_char(), Ok(b'z'));
        assert_eq!(s.next_char(), Err(ReadError::EndOfFile));
    }
}
