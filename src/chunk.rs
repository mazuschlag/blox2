use std::fmt;

use crate::value::*;

#[derive(Debug, Clone)]
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
        if line >= self.lines.len() {
            self.lines.push(1);
        } else {
            self.lines[line - 1] += 1;
        }
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        for (instruction_number, byte) in self.code.iter().enumerate() {
            self.disassemble_instruction(instruction_number, byte);
        }

        println!("==\\ {name} ==")
    }

    pub fn read_op(&self, index: usize) -> &OpCode {
        self.code
            .get(index)
            .expect("Operation read error - instruction index is out-of-bounds")
    }

    pub fn read_constant(&self, index: usize) -> &Value {
        self.constants
            .get(index)
            .expect("Constant read error - index for constant is out-of-bounds")
    }

    pub fn disassemble_instruction(&self, offset: usize, instruction: &OpCode) {
        print!("{:04} ", offset);
        let current_line = self.get_line(offset);
        if offset > 0 && current_line == self.get_line(offset - 1) {
            print!("    | ");
        } else {
            print!("{number:>width$} ", number = current_line, width = 5);
        }

        match instruction {
            OpCode::Constant(index) => {
                let value = self.constants[*index];
                println!("{instruction} '{value}'");
            }
            _ => println!("{instruction}"),
        };
    }

    pub fn get_line(&self, offset: usize) -> usize {
        let mut line_counter = self.lines[0];
        let mut current_index = 1;
        while line_counter < offset && current_index < self.lines.len() {
            line_counter += self.lines[current_index];
            current_index += 1;
        }
        current_index
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant(usize),
    Nil,
    True,
    False,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(index) => {
                write!(f, "CONSTANT {number:>width$}", number = index, width = 16)
            }
            Self::Nil => write!(f, "NIL"),
            Self::True => write!(f, "TRUE"),
            Self::False => write!(f, "FALSE"),
            Self::Add => write!(f, "ADD"),
            Self::Subtract => write!(f, "SUBTRACT"),
            Self::Multiply => write!(f, "MULTIPLY"),
            Self::Divide => write!(f, "DIVIDE"),
            Self::Not => write!(f, "NOT"),
            Self::Negate => write!(f, "NEGATE"),
            Self::Return => write!(f, "RETURN"),
        }
    }
}
