use std::io::{self, stdin, stdout, Write};
use std::rc::Rc;

use lisp::reader::Reader;

fn prompt(s: &str) -> io::Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write(s.as_bytes())?;
    stdout.flush()
}

fn main() {
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut reader = lisp::reader::InputStream::from_reader(stdin);

    let env = lisp::env::Env::global_env();

    loop {
        prompt("LISP> ").unwrap();
        match reader.read_ahead() {
            Ok(x) => match lisp::eval::eval(x, Rc::clone(&env)) {
                Ok(result) => println!("{}", result),
                Err(e) => println!("{}", e),
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
