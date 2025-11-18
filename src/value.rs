use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Number(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}
