use std::io::{self, stdin, stdout, BufRead, BufReader, Write};
use std::rc::Rc;

fn prompt(s: &str) -> io::Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write(s.as_bytes())?;
    stdout.flush()
}

fn main() {
    let stdin = stdin();
    let stdin = stdin.lock();
    let stdin = BufReader::new(stdin);
    let mut lines = stdin.lines();

    let env = lisp::env::Env::global_env();

    loop {
        prompt("LISP> ").unwrap();
        if let Some(Ok(input)) = lines.next() {
            match lisp::read::read_from_string(&input) {
                Ok((obj, _pos)) => match lisp::eval::eval(obj, Rc::clone(&env)) {
                    Ok(result) => println!("{}", result),
                    Err(e) => println!("{}", e),
                },
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }
}
