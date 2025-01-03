use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use lexer::Lexer;
use lexer::TokenType;
pub mod lexer;

fn run_lexer(filename: &str) -> i32 {
    let file_contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(_) => {
            writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
            return 1;
        }
    };

    if file_contents.is_empty() {
        println!("EOF  null");
        return 0;
    }

    let mut lexer = Lexer::new();
    let tokens = lexer.lex(&file_contents);

    for token in tokens {
        match token.token_type {
            TokenType::STRING(ref s) => println!("STRING \"{}\" {}", s, s),
            TokenType::NUMBER(org_val, val) => println!("NUMBER {org_val} {val}"),
            TokenType::Eof => println!("EOF  null"),
            _ => println!("{:?} {} null", token.token_type, token.lexeme),
        }
    }

    if lexer.had_error() {
        65
    } else {
        0
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        exit(1);
    }

    match args[1].as_str() {
        "tokenize" => exit(run_lexer(&args[2])),
        cmd => {
            writeln!(io::stderr(), "Unknown command: {}", cmd).unwrap();
            exit(1);
        }
    }
}
