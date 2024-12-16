use crate::{
    chunk::*,
    scanner::*,
};

pub struct Compiler {
    scanner: Scanner,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self {
            scanner: Scanner::new(source),
        }
    }
    pub fn compile(&mut self) -> Result<Chunk, String> {
        Ok(Chunk::new())
    }
}
