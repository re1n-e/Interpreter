use crate::lexer::{return_tokens, Literal, Token, TokenType};
use crate::parse::{Expr, Parser, Stmt};
use std::fmt;
use std::fs;
use std::io::{self, Write};

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{}", value),
            Value::String(value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{:?}", value),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub token: Token,
    pub line: usize,
}

type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Evaluate;

impl Evaluate {
    pub fn new() -> Self {
        Evaluate
    }

    pub fn visit_expression_stmt(&self, expr: &Expr) -> Result<Value> {
        self.evaluate(expr)
    }

    pub fn visit_print_stmt(&self, expr: &Expr) {
        let value = self.evaluate(expr);
        match value {
            Ok(v) => println!("{}", v),
            Err(error) => {
                writeln!(
                    io::stderr(),
                    "[line {}] Runtime Error: {}",
                    error.line,
                    error.message
                )
                .unwrap();
                std::process::exit(70)
            }
        }
    }

    pub fn evaluate(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal { value } => match value {
                Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                Literal::Number(b) => Ok(Value::Number(*b)),
                Literal::String(b) => Ok(Value::String(b.clone())),
                Literal::None => Ok(Value::Nil),
                _ => Ok(Value::Nil),
            },
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::MINUS => {
                        if let Value::Number(n) = right {
                            Ok(Value::Number(-n))
                        } else {
                            Err(RuntimeError {
                                message: "Operand must be a number.".to_string(),
                                token: operator.clone(),
                                line: operator.line,
                            })
                        }
                    }
                    TokenType::BANG => Ok(Value::Boolean(!self.is_truthy(&right))),
                    _ => Err(RuntimeError {
                        message: "Invalid unary operator.".to_string(),
                        token: operator.clone(),
                        line: operator.line,
                    }),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::MINUS => {
                        self.number_operation(&left, &right, |a, b| a - b, operator)
                    }
                    TokenType::SLASH => {
                        self.number_operation(&left, &right, |a, b| a / b, operator)
                    }
                    TokenType::STAR => self.number_operation(&left, &right, |a, b| a * b, operator),
                    TokenType::PLUS => match (&left, &right) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => {
                            Ok(Value::String(format!("{}{}", a, b)))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be two numbers or two strings.".to_string(),
                            token: operator.clone(),
                            line: operator.line,
                        }),
                    },
                    TokenType::GREATER => {
                        self.number_operation(&left, &right, |a, b| a > b, operator)
                    }
                    TokenType::GREATER_EQUAL => {
                        self.number_operation(&left, &right, |a, b| a >= b, operator)
                    }
                    TokenType::LESS => self.number_operation(&left, &right, |a, b| a < b, operator),
                    TokenType::LESS_EQUAL => {
                        self.number_operation(&left, &right, |a, b| a <= b, operator)
                    }
                    TokenType::BANG_EQUAL => Ok(Value::Boolean(!self.is_equal(&left, &right))),
                    TokenType::EQUAL_EQUAL => Ok(Value::Boolean(self.is_equal(&left, &right))),
                    _ => Err(RuntimeError {
                        message: "Invalid binary operator.".to_string(),
                        token: operator.clone(),
                        line: operator.line,
                    }),
                }
            }
        }
    }

    fn number_operation<T, F>(
        &self,
        left: &Value,
        right: &Value,
        op: F,
        operator: &Token,
    ) -> Result<Value>
    where
        F: Fn(f64, f64) -> T,
        T: Into<Value>,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(op(*a, *b).into()),
            _ => Err(RuntimeError {
                message: "Operands must be numbers.".to_string(),
                token: operator.clone(),
                line: operator.line,
            }),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

pub fn evaluate(filename: &str) {
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
    let Evaluate = Evaluate::new();
    let statement = parser.parse();
    for stmt in statement {
        match stmt {
            Stmt::Expression(expr) => match Evaluate.visit_expression_stmt(&expr) {
                Ok(_) => (),
                Err(error) => {
                    writeln!(
                        io::stderr(),
                        "[line {}] Runtime Error: {}",
                        error.line,
                        error.message
                    )
                    .unwrap();
                    std::process::exit(70)
                }
            },
            Stmt::Print(expr) => Evaluate.visit_print_stmt(&expr),
        }
    }
}
