use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(usize),
}

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

    pub fn as_obj(&self) -> usize {
        match self {
            Self::Obj(index) => *index,
            _ => panic!("Value is not of type 'Obj'."),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Obj(o) => write!(f, "{o}"),
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

#[derive(Debug, Clone)]
pub enum Obj {
    Str(String),
    Ident(String),
}

impl Obj {
    pub fn name(&self) -> &String {
        match self {
            Self::Str(s) | Self::Ident(s) => s,
        }
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Str(s) | Self::Ident(s) => write!(f, "{s}"),
        }
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Str(s), Self::Str(o)) | (Self::Ident(s), Self::Ident(o)) => s == o,
            (_, _) => false,
        }
    }
}

impl Eq for Obj {}
