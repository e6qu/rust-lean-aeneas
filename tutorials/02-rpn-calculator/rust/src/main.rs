// Tutorial 02: RPN Calculator — CLI Shell
//
// This is the *unverified* shell. It handles I/O (reading from stdin,
// printing results) and calls into the verified pure core in lib.rs.
// Aeneas only translates the library; this file is not part of the proof.

use rpn_calc::{EvalError, evaluate, tokenize_word};

fn main() {
    let mut input = String::new();
    println!("RPN Calculator — enter an expression (e.g., 3 4 + 2 *):");

    if std::io::stdin().read_line(&mut input).is_err() {
        eprintln!("Error: could not read input");
        std::process::exit(1);
    }

    let mut tokens = Vec::new();
    for word in input.split_whitespace() {
        match tokenize_word(word.as_bytes()) {
            Ok(token) => tokens.push(token),
            Err(EvalError::InvalidToken) => {
                eprintln!("Error: invalid token '{}'", word);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    match evaluate(&tokens) {
        Ok(result) => println!("Result: {}", result),
        Err(EvalError::DivisionByZero) => eprintln!("Error: division by zero"),
        Err(EvalError::StackUnderflow) => eprintln!("Error: not enough operands"),
        Err(EvalError::TooManyValues) => eprintln!("Error: too many values on stack"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
