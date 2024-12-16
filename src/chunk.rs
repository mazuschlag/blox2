use std::fmt;

use crate::common::*;

#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    Constant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

pub struct Chunk {
    code: Vec<OpCode>,
    lines: Vec<usize>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    #[allow(dead_code)]
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (offset, byte) in self.code.iter().enumerate() {
            self.disassemble_instruction(offset, byte);
        }
    }

    pub fn read_op(&self, index: usize) -> &OpCode {
        assert!(index < self.code.len(), "Index for operation is out-of-bounds");
        &self.code[index]
    }

    pub fn read_constant(&self, index: usize) -> &Value {
        assert!(index < self.constants.len(), "Index for constant is out-of-bounds");
        &self.constants[index]
    }

    pub fn disassemble_instruction(&self, offset: usize, instruction: &OpCode) {
        print!("{:04} ", offset);
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("    | ");
        } else {
            print!(
                "{number:>width$} ",
                number = self.get_line(offset),
                width = 5
            );
        }
        match instruction {
            OpCode::Constant(index) => {
                let value = self.constants[*index];
                println!("{instruction} '{value}'");
            }
            _ => println!("{instruction}"),
        };
    }

    fn get_line(&self, offset: usize) -> usize {
        let mut line_counter = self.lines[0];
        let mut current_index = 1;
        while line_counter <= offset && current_index < self.lines.len() {
            line_counter += self.lines[current_index];
            current_index += 1;
        }
        current_index
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(index) => {
                write!(f, "CONSTANT {number:>width$}", number = index, width = 16)
            },
            Self::Add => write!(f, "ADD"),
            Self::Subtract => write!(f, "SUBTRACT"),
            Self::Multiply => write!(f, "MULTIPLY"),
            Self::Divide => write!(f, "DIVIDE"),
            Self::Negate => write!(f, "NEGATE"),
            Self::Return => write!(f, "RETURN"),
        }
    }
}