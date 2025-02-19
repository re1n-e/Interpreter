use crate::lexer::{Lexer, Token, TokenType};
use std::fs;
use std::io::{self, Write};
use std::any::Any;

fn parse_number(val: &str, token: Token) -> Result<Expr, ParseError> {
    match val.parse::<f64>() {
        Ok(num) => Ok(Expr::Literal {
            value: Box::new((num, val.to_string())),
        }),
        Err(_) => Err(ParseError {
            token,
            message: format!("Invalid number: {}", val),
        }),
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Box<dyn Any>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    fn ast_print(&self) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    left.ast_print(),
                    right.ast_print()
                )
            }
            Expr::Grouping { expression } => {
                format!("(group {})", expression.ast_print())
            }
            Expr::Literal { value } => {
                if let Some(s) = value.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(n) = value.downcast_ref::<f64>() {
                    n.to_string()
                } else if let Some(b) = value.downcast_ref::<bool>() {
                    b.to_string()
                } else if value.downcast_ref::<()>().is_some() {
                    "nil".to_string()
                } else if let Some((_, n)) = value.downcast_ref::<(f64, String)>() {
                    n.to_string()
                } else {
                    "Unknown Literal".to_string()
                }
            }
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, right.ast_print())
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    token: Token,
    message: String,
}

pub struct Parser<'a> {
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
                    Ok(Expr::Literal {
                        value: Box::new(false),
                    })
                }
                TokenType::TRUE => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Box::new(true),
                    })
                }
                TokenType::NIL => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Box::new(()),
                    })
                }
                TokenType::NUMBER(_, val) => {
                    let val = val.clone();
                    self.advance();
                    parse_number(&val, token.clone()) 
                }
                TokenType::STRING(s) => {
                    let s = s.clone();
                    self.advance();
                    Ok(Expr::Literal { value: Box::new(s) })
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
                eprintln!(
                    "Parse error at line {}: {}",
                    error.token.line, error.message
                );
                self.had_error = true;
                None
            }
        }
    }
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

    let mut parser = Parser::new(&mut tokens);

    match parser.parse() {
        Some(expr) => {
            println!("{}", expr.ast_print());
            if lexer.had_error() || parser.had_error() {
                65
            } else {
                0
            }
        }
        None => 65,
    }
}
