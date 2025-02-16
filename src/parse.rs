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
        value: String,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
struct ParseError {
    token: Token,
    message: String,
}

struct Parser<'a> {
    tokens: &'a mut std::iter::Peekable<std::vec::IntoIter<Token>>,
    current: usize,
    had_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> Self {
        Parser { 
            tokens, 
            current: 0,
            had_error: false,
        }
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn advance(&mut self) -> Option<Token> {
        self.current += 1;
        self.tokens.next()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_op(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while let Some(op) = self.match_op(vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.match_op(vec![TokenType::MINUS, TokenType::PLUS]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_op(vec![TokenType::SLASH, TokenType::STAR]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(op) = self.match_op(vec![TokenType::BANG, TokenType::MINUS]) {
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.peek().cloned() {
            match &token.token_type {
                TokenType::FALSE => {
                    self.advance();
                    Ok(Expr::Literal { value: String::from("false") })
                }
                TokenType::TRUE => {
                    self.advance();
                    Ok(Expr::Literal { value: String::from("true") })
                }
                TokenType::NIL => {
                    self.advance();
                    Ok(Expr::Literal { value: String::from("nil") })
                }
                TokenType::NUMBER(_, val) => {
                    let val = val.clone();
                    self.advance();
                    Ok(Expr::Literal { value: val.to_string() })
                }
                TokenType::STRING(s) => {
                    let s = s.clone();
                    self.advance();
                    Ok(Expr::Literal { value: s })
                }
                TokenType::LEFT_PAREN => {
                    self.advance();
                    let expr = self.expression()?;
                    
                    match self.peek() {
                        Some(t) if t.token_type == TokenType::RIGHT_PAREN => {
                            self.advance();
                            Ok(Expr::Grouping {
                                expression: Box::new(expr),
                            })
                        }
                        Some(t) => Err(ParseError {
                            token: t.clone(),
                            message: String::from("Expected ')' after expression."),
                        }),
                        None => Err(ParseError {
                            token: token.clone(),
                            message: String::from("Unexpected end of input, expected ')'"),
                        }),
                    }
                }
                _ => Err(ParseError {
                    token: token.clone(),
                    message: String::from("Expected expression."),
                }),
            }
        } else {
            Err(ParseError {
                token: Token {
                    token_type: TokenType::Eof,
                    lexeme: String::from(""),
                    line: 0,
                },
                message: String::from("Unexpected end of input."),
            })
        }
    }

    fn match_op(&mut self, ops: Vec<TokenType>) -> Option<Token> {
        if let Some(token) = self.peek() {
            if ops.contains(&token.token_type) {
                return self.advance();
            }
        }
        None
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(error) => {
                eprintln!("Parse error at line {}: {}", 
                    error.token.line, 
                    error.message);
                self.had_error = true;
                None
            }
        }
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
