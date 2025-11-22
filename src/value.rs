use std::{fmt, rc::Rc};

#[derive(Debug, Clone)]
pub enum Obj {
    Str(Rc<Obj>, String),
    Unit,
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Str(_, s) => write!(f, "'{s}'"),
            Self::Unit => write!(f, "()"),
        }
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Str(_, a), Self::Str(_, b)) => a == b,
            (Self::Unit, Self::Unit) => true,
            (_, _) => false,
        }
    }
}

impl Eq for Obj {}

impl Iterator for Obj {
    type Item = Rc<Obj>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Str(obj, _) => Some(Rc::clone(obj)),
            Self::Unit => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Rc<Obj>),
    Identifier(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Obj(o) => write!(f, "{o}"),
            Self::Identifier(i) => write!(f, "{i}"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Obj(a), Value::Obj(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Nil | Self::Bool(false) => true,
            _ => false,
        }
    }

    pub fn name(&self) -> String {
        if let Value::Identifier(name) = self {
            return name.clone();
        }

        panic!("Value does not have a name");
    }
}
