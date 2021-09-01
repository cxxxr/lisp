mod lisp;
use std::io::{stdin, BufRead, BufReader};

fn main() {
    let stdin = stdin();
    let stdin = stdin.lock();
    let stdin = BufReader::new(stdin);
    let mut lines = stdin.lines();

    loop {
        if let Some(Ok(input)) = lines.next() {
            let r = lisp::reader::read_from_string(&input);
            match r {
                Ok((obj, _pos)) => {
                    println!("{}", obj);
                }
                Err(e) => {
                    println!("error: {:?}", e);
                }
            }
        }
    }
}
