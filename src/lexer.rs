use std::fs;
use std::io::{self, Write};
use std::iter::Peekable;
use std::str::Chars;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Grouping tokens
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,

    // Single-character tokens
    STAR,
    DOT,
    COMMA,
    PLUS,
    MINUS,
    SEMICOLON,
    SLASH,
    COMMENT,

    // One or two character tokens
    EQUAL,
    EQUAL_EQUAL,
    BANG,
    BANG_EQUAL,
    LESS,
    LESS_EQUAL,
    GREATER,
    GREATER_EQUAL,

    // Literals
    STRING(String),

    // Number
    NUMBER(String, String),

    // Special tokens
    IDENTIFIER(String),

    // IDENTIFIER
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

pub struct Lexer {
    had_error: bool,
    line: usize,
}

fn keywords(key: &str) -> Option<TokenType> {
    match key {
        "and" => Some(TokenType::AND),
        "class" => Some(TokenType::CLASS),
        "else" => Some(TokenType::ELSE),
        "false" => Some(TokenType::FALSE),
        "for" => Some(TokenType::FOR),
        "fun" => Some(TokenType::FUN),
        "if" => Some(TokenType::IF),
        "nil" => Some(TokenType::NIL),
        "or" => Some(TokenType::OR),
        "print" => Some(TokenType::PRINT),
        "return" => Some(TokenType::RETURN),
        "super" => Some(TokenType::SUPER),
        "this" => Some(TokenType::THIS),
        "true" => Some(TokenType::TRUE),
        "var" => Some(TokenType::VAR),
        "while" => Some(TokenType::WHILE),
        _ => None,
    }
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            had_error: false,
            line: 0,
        }
    }

    pub fn lex(&mut self, source: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut char_iter = source.chars().peekable();
        self.line = 0;

        let mut current_line = 1;

        while let Some(ch) = char_iter.next() {
            if ch == '\n' {
                self.line += 1;
                current_line = self.line + 1;
            }

            if let Some(token) = self.scan_token(ch, &mut char_iter, current_line) {
                tokens.push(token)
            }
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::from("EOF"),
            line: current_line,
        });

        tokens
    }

    fn scan_token(&mut self, ch: char, chars: &mut Peekable<Chars>, line: usize) -> Option<Token> {
        match ch {
            '(' => Some(self.make_token(TokenType::LEFT_PAREN, "(", line)),
            ')' => Some(self.make_token(TokenType::RIGHT_PAREN, ")", line)),
            '{' => Some(self.make_token(TokenType::LEFT_BRACE, "{", line)),
            '}' => Some(self.make_token(TokenType::RIGHT_BRACE, "}", line)),
            '*' => Some(self.make_token(TokenType::STAR, "*", line)),
            '.' => Some(self.make_token(TokenType::DOT, ".", line)),
            ',' => Some(self.make_token(TokenType::COMMA, ",", line)),
            '+' => Some(self.make_token(TokenType::PLUS, "+", line)),
            '-' => Some(self.make_token(TokenType::MINUS, "-", line)),
            ';' => Some(self.make_token(TokenType::SEMICOLON, ";", line)),

            // Two-character tokens
            '=' => self.match_next(
                chars,
                '=',
                '=',
                TokenType::EQUAL_EQUAL,
                TokenType::EQUAL,
                line,
            ),
            '!' => self.match_next(
                chars,
                '!',
                '=',
                TokenType::BANG_EQUAL,
                TokenType::BANG,
                line,
            ),
            '<' => self.match_next(
                chars,
                '<',
                '=',
                TokenType::LESS_EQUAL,
                TokenType::LESS,
                line,
            ),
            '>' => self.match_next(
                chars,
                '>',
                '=',
                TokenType::GREATER_EQUAL,
                TokenType::GREATER,
                line,
            ),
            // Comments
            '/' => {
                if chars.peek() == Some(&'/') {
                    chars.next();

                    while let Some(ch) = chars.peek() {
                        if *ch == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    None
                } else {
                    Some(self.make_token(TokenType::SLASH, "/", line))
                }
            }

            // String literals
            '"' => self.scan_string(chars, line),

            // Number
            '0'..='9' => self.scan_num(ch, chars, line),

            // Whitespace
            ch if ch.is_whitespace() => None,

            id if id == '_' || id.is_alphanumeric() => self.scan_identifier(id, chars, line),

            // Unexpected characters
            _ => {
                self.error(line, &format!("Unexpected character: {}", ch));
                None
            }
        }
    }

    fn scan_string(&mut self, chars: &mut Peekable<Chars>, line: usize) -> Option<Token> {
        let mut value = String::new();
        let mut line = line;
        while chars.peek().is_some() {
            if let Some(ch) = chars.peek() {
                match ch {
                    '"' => {
                        chars.next();
                        return Some(Token {
                            token_type: TokenType::STRING(value.clone()),
                            lexeme: format!("\"{}\"", value),
                            line,
                        });
                    }
                    '\n' => {
                        line += 1;
                        value.push(*ch);
                    }
                    _ => value.push(*ch),
                }
            }
            chars.next();
        }
        self.error(line, "Unterminated string.");
        None
    }

    fn scan_num(&mut self, num: char, chars: &mut Peekable<Chars>, line: usize) -> Option<Token> {
        let mut value = String::from(num);
        let mut org_value = String::from(num);
        let mut zeroes = String::new();
        let mut deci = false;
        while let Some(ch) = chars.peek() {
            match ch {
                '0' => {
                    if deci {
                        zeroes.push('0');
                    } else {
                        value.push(*ch);
                    }
                }
                '1'..='9' => {
                    if !zeroes.is_empty() {
                        value.push_str(&zeroes);
                        zeroes.clear();
                    }
                    value.push(*ch);
                }
                '.' => {
                    value.push(*ch);
                    deci = true;
                }
                _ => break,
            }
            org_value.push(*ch);
            chars.next();
        }

        if !deci {
            value.push_str(&".0");
        } else if value.ends_with('.') {
            value.push('0');
        }

        Some(Token {
            token_type: TokenType::NUMBER(org_value.clone(), value.clone()),
            lexeme: format!("\"{}\"", value),
            line,
        })
    }

    fn scan_identifier(
        &mut self,
        num: char,
        chars: &mut Peekable<Chars>,
        line: usize,
    ) -> Option<Token> {
        let mut identifier = String::from(num);
        while let Some(ch) = chars.peek() {
            match ch {
                ch if ch.is_alphanumeric() || ch == &'_' => identifier.push(*ch),
                _ => break,
            }
            chars.next();
        }

        if let Some(reserved) = keywords(&identifier) {
            Some(Token {
                token_type: reserved,
                lexeme: format!("{}", identifier),
                line,
            })
        } else {
            Some(Token {
                token_type: TokenType::IDENTIFIER(identifier.clone()),
                lexeme: format!("\"{}\"", identifier),
                line,
            })
        }
    }

    fn match_next(
        &self,
        chars: &mut Peekable<Chars>,
        current: char,
        expected: char,
        double_type: TokenType,
        single_type: TokenType,
        line: usize,
    ) -> Option<Token> {
        let (token_type, lexeme) = if chars.peek() == Some(&expected) {
            chars.next();
            (double_type, format!("{}{}", current, expected))
        } else {
            (single_type, current.to_string())
        };

        Some(self.make_token(token_type, &lexeme, line))
    }

    fn make_token(&self, token_type: TokenType, lexeme: &str, line: usize) -> Token {
        Token {
            token_type,
            lexeme: String::from(lexeme),
            line,
        }
    }

    fn error(&mut self, line: usize, message: &str) {
        eprintln!("[line {}] Error: {}", line, message);
        self.had_error = true;
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }
}

pub fn run_lexer(filename: &str) -> i32 {
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
            TokenType::IDENTIFIER(iden) => println!("IDENTIFIER {} null", iden),
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
