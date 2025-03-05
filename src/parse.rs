use crate::lexer::{return_tokens, Literal, Token, TokenType};
use std::fs;
use std::io::{self, Write};

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
        value: Literal,
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
            Expr::Grouping { expression } => format!("(group {})", expression.ast_print()),
            Expr::Literal { value } => match value {
                Literal::String(s) => s.clone(),
                Literal::Number(n) => format!("{:?}", n),
                Literal::Boolean(b) => b.to_string(),
                Literal::None => "nil".to_string(),
                _ => "Unknown Literal".to_string(),
            },
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

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    had_error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            had_error: false,
        }
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

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn peek(&self) -> Option<Token> {
        if self.current < self.tokens.len() {
            return Some(self.tokens[self.current].clone());
        }
        None
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Option<ParseError> {
        if let Some(peek) = self.peek() {
            if peek.token_type == token_type {
                self.advance();
                return None;
            }
        }
        self.had_error = true;
        Some(ParseError {
            token: self.peek().unwrap(),
            message: message.to_string(),
        })
    }

    fn match_token(&mut self, token_types: Vec<TokenType>) -> Option<Token> {
        if let Some(peek_token) = self.peek() {
            for token_type in token_types {
                if token_type == peek_token.token_type {
                    self.advance();
                    return Some(peek_token);
                }
            }
        }
        None
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while let Some(op) = self.match_token(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while let Some(op) = self.match_token(vec![
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
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while let Some(op) = self.match_token(vec![TokenType::MINUS, TokenType::PLUS]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while let Some(op) = self.match_token(vec![TokenType::SLASH, TokenType::STAR]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(op) = self.match_token(vec![TokenType::BANG, TokenType::MINUS]) {
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.peek() {
            match token.token_type {
                TokenType::FALSE => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Literal::Boolean(false),
                    })
                }
                TokenType::TRUE => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Literal::Boolean(true),
                    })
                }
                TokenType::NIL => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Literal::None,
                    })
                }
                TokenType::NUMBER => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Literal::Number(token.lexeme.parse::<f64>().unwrap()),
                    })
                }
                TokenType::STRING => {
                    self.advance();
                    Ok(Expr::Literal {
                        value: Literal::String(format!("{}", token.literal)),
                    })
                }
                TokenType::LEFT_PAREN => {
                    self.advance();
                    let expr = self.expression()?;
                    if let Some(err) =
                        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.")
                    {
                        return Err(err);
                    }
                    Ok(Expr::Grouping {
                        expression: Box::new(expr),
                    })
                }
                _ => Err(ParseError {
                    token: token.clone(),
                    message: String::from("Expected expression."),
                }),
            }
        } else {
            Err(ParseError {
                token: Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 0,
                    literal: Literal::None,
                },
                message: String::from("Unexpected end of input."),
            })
        }
    }
}

pub fn run_parser(filename: &str) {
    let file_contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(_) => {
            writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
            return;
        }
    };

    if file_contents.is_empty() {
        println!("EOF  null");
        return;
    }

    let mut parser = Parser::new(return_tokens(&file_contents));
    match parser.parse() {
        Some(expr) => {
            println!("{}", expr.ast_print());
            if parser.had_error {
                std::process::exit(65)
            } else {
                std::process::exit(0)
            }
        }
        None => std::process::exit(65),
    }
}
