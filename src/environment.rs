use crate::{
    evaluate::RuntimeError,
    lexer::{Literal, Token},
};
use std::collections::HashMap;
pub struct Environment {
    map: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            map: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, value: Literal) {
        self.map.insert(name, value);
    }

    fn get(&mut self, name: Token) -> Result<Literal, RuntimeError> {
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
