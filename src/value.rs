use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_equal(&self, value: &Value) -> bool {
        match (self, value) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            _ => false,
        }
    }
}
