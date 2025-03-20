use crate::lexer::Token;
use crate::evaluate::{RuntimeError, Value};
use std::collections::HashMap;

pub struct Environment {
    map: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            map: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.map.insert(name, value);
    }

    pub fn get(&self, name: Token) -> Result<Value, RuntimeError> {
        match self.map.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => Err(RuntimeError {
                message: format!("Undefined variable {}.", name.lexeme),
                line: name.line,
                token: name,
            }),
        }
    }
}
