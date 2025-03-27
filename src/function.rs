use crate::lexer::Token;
use crate::{
    environment::Environment,
    evaluate::{Evaluate, RuntimeError, Value},
    parse::Stmt,
};
use std::cell::RefCell;
use std::rc::Rc;
pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Evaluate,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
    fn to_string(&self) -> String;
}

use std::time::{SystemTime, UNIX_EPOCH};
pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Evaluate,
        _arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        Ok(Value::Number((since_epoch.as_millis() / 1000) as f64))
    }

    fn to_string(&self) -> String {
        "<native fn>".to_string()
    }
}

pub struct LoxFunction {
    name: Token,
    parameter: Vec<Token>,
    body: Vec<Stmt>,
}

impl LoxFunction {
    pub fn new(name: Token, parameter: Vec<Token>, body: Vec<Stmt>) -> Self {
        LoxFunction {
            name,
            parameter,
            body,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.parameter.len()
    }

    fn call(
        &self,
        interpreter: &mut Evaluate,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let mut environment = Environment::from_enclosing(Rc::clone(&interpreter.globals));

        for i in 0..self.parameter.len() {
            environment.define(self.parameter[i].lexeme.clone(), arguments[i].clone());
        }
        interpreter.execute_block(self.body.clone(), Rc::new(RefCell::new(environment)));

        Ok(Value::Nil)
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}
