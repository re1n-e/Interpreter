use crate::lexer::{Lexer, Token, TokenType};
use std::fs;
use std::io::{self, Write};

#[derive(Debug)]
enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: f64,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

struct Parser<'a> {
    tokens: &'a mut std::iter::Peekable<std::vec::IntoIter<Token>>,
    current: usize,
}

// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut node: Expr = self.comparison();
        while let Some(op) = self.match_op(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let right = self.comparison();
            node = Expr::Binary { left: Box::new(node), operator: op, right: Box::new(right) }
        }
        node
    }

    fn comparison(&mut self) -> Expr {}

    fn match_op(&mut self, ops: Vec<TokenType>) -> Option<Token> {
        if let Some(token) = self.peek() {
            if ops.contains(&token.token_type) {
                return self.advance()
            }
        }
        None
    }
}

fn parse_group(tokens: &mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> String {
    let mut group_tokens: Vec<String> = Vec::new();
    let mut depth = 1;

    group_tokens.push(String::from("(group"));

    while let Some(token) = tokens.next() {
        match token.token_type {
            TokenType::LEFT_PAREN => {
                depth += 1;
                group_tokens.push(String::from("(group"));
            }
            TokenType::RIGHT_PAREN => {
                depth -= 1;
                if let Some(last) = group_tokens.pop() {
                    group_tokens.push(format!("{last})"));
                }

                if depth == 0 {
                    break;
                }
            }
            TokenType::STRING(s) => group_tokens.push(s),
            TokenType::NUMBER(_, val) => group_tokens.push(val.to_string()),
            TokenType::IDENTIFIER(id) => group_tokens.push(id),
            _ => group_tokens.push(token.lexeme),
        }
    }

    group_tokens.join(" ")
}

pub fn parse(filename: &str) -> i32 {
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
    let mut tokens = lexer.lex(&file_contents).into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token.token_type {
            TokenType::Eof => break,
            TokenType::STRING(ref s) => println!("{}", s),
            TokenType::NUMBER(_, val) => println!("{val}"),
            TokenType::LEFT_PAREN => {
                let group_result = parse_group(&mut tokens);
                println!("{}", group_result);
            }
            TokenType::BANG => {
                if let Some(t) = tokens.next() {
                    println!("(! {})", t.lexeme);
                }
            }
            TokenType::MINUS => {
                if let Some(t) = tokens.next() {
                    match t.token_type {
                        TokenType::NUMBER(_, val) => println!("(- {val})"),
                        _ => println!("(- {})", t.lexeme),
                    }
                }
            }
            _ => println!("{}", token.lexeme),
        }
    }

    if lexer.had_error() {
        65
    } else {
        0
    }
}
