use crate::evaluate::{RuntimeError, Value};
use crate::lexer::Token;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn enclosing(&mut self) {
        self.enclosing = Some(Box::new(Environment::new()));
    }

    pub fn assign(&mut self, name: Token, value: Value) -> Result<(), RuntimeError> {
        if let Some(_) = self.values.get(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
        }
        match &mut self.enclosing {
            Some(enclose) => enclose.assign(name, value),
            None => Err(RuntimeError {
                message: format!("Undefined variable '{}'.", name.lexeme),
                line: name.line,
                token: name,
            }),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Token) -> Result<Value, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => match &self.enclosing {
                Some(enclose) => enclose.get(name),
                None => Err(RuntimeError {
                    message: format!("Undefined variable '{}'.", name.lexeme),
                    line: name.line,
                    token: name,
                }),
            },
        }
    }
}
