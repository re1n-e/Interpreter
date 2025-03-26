use crate::evaluate::{Evaluate, Value, RuntimeError};
pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Evaluate, arguments: Vec<Value>) -> Result<Value, RuntimeError>;
    fn to_string(&self) -> String;
}

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Evaluate, _arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        Ok(Value::Number(since_epoch.as_millis() as f64))
    }

    fn to_string(&self) -> String {
        "<native fn>".to_string()
    }
}
