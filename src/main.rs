use std::env;
use std::io::{self, Write};
use std::process::exit;
pub mod evaluate;
pub mod lexer;
pub mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        exit(1);
    }
    match args[1].as_str() {
        "tokenize" => exit(lexer::run_lexer(&args[2])),
        "parse" => exit(parse::parse(&args[2])),
        "evaluate" => exit(evaluate::evaluate(&args[2])),
        "run" => exit(parse::run(&args[2])),
        cmd => {
            writeln!(io::stderr(), "Unknown command: {}", cmd).unwrap();
            exit(1);
        }
    }
}
