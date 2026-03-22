// Tutorial 03: Infix Calculator — CLI wrapper
//
// This file is NOT verified by Aeneas. It provides a simple command-line
// interface to the verified calculator core.

use infix_calc::{eval, lex, parse};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    write!(stdout, "Infix Calculator (type an expression, e.g. '(2 + 3) * 4')\n").unwrap();
    write!(stdout, "Press Ctrl-D to exit.\n\n").unwrap();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let tokens = match lex(line.as_bytes()) {
            Ok(t) => t,
            Err(e) => {
                writeln!(stdout, "Lex error: {:?}", e).unwrap();
                continue;
            }
        };

        let expr = match parse(&tokens) {
            Ok(e) => e,
            Err(e) => {
                writeln!(stdout, "Parse error: {:?}", e).unwrap();
                continue;
            }
        };

        match eval(&expr) {
            Ok(result) => writeln!(stdout, "= {}", result).unwrap(),
            Err(e) => writeln!(stdout, "Eval error: {:?}", e).unwrap(),
        }
    }
}
