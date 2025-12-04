use std::fmt;

use crate::arena::*;
use crate::value::*;

#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<Op>,
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

    pub fn write(&mut self, byte: Op, line: usize) {
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

    pub fn disassemble(&self, name: &str, objects: &Arena<Obj>) {
        println!("== {name} ==");
        print!("Constants: ");
        for (constant_index, constant) in self.constants.iter().enumerate() {
            match constant {
                Value::Obj(index) => print!("{constant_index}:[ {} ] ", objects.get(*index)),
                _ => print!("{constant_index}:[ {constant} ] "),
            }
        }

        println!();
        for (instruction_number, byte) in self.code.iter().enumerate() {
            self.disassemble_instruction(instruction_number, byte, objects);
        }

        println!("==\\ {name} ==")
    }

    pub fn read_op(&self, index: usize) -> &Op {
        self.code
            .get(index)
            .expect("Operation read error - instruction index is out-of-bounds")
    }

    pub fn read_constant(&self, index: usize) -> &Value {
        self.constants
            .get(index)
            .expect("Constant read error - index for constant is out-of-bounds")
    }

    pub fn disassemble_instruction(&self, offset: usize, instruction: &Op, objects: &Arena<Obj>) {
        print!("{:04} ", offset);
        let current_line = self.get_line(offset);
        if offset > 0 && current_line == self.get_line(offset - 1) {
            print!("    | ");
        } else {
            print!("{number:>width$} ", number = current_line, width = 5);
        }

        match instruction {
            Op::Constant(index) => {
                let value = &self.constants[*index];
                match value {
                    Value::Obj(index) => println!("{instruction} '{}'", objects.get(*index)),
                    _ => println!("{instruction} '{value}'"),
                }
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

    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    pub fn get_op(&self, index: usize) -> &Op {
        self.code.get(index).expect("Index for op is out of bounds")
    }

    pub fn get_op_mut(&mut self, index: usize) -> &mut Op {
        self.code.get_mut(index).expect("Index for op is out of bounds")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Constant(usize),
    Nil,
    True,
    False,
    Pop,
    DefineGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    GetLocal(usize),
    SetLocal(usize),
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    JumpIfFalse(usize),
    Jump(usize),
    Return,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Constant(index) => {
                write!(f, "CONSTANT {number:>width$}", number = index, width = 16)
            }
            Self::Nil => write!(f, "NIL"),
            Self::True => write!(f, "TRUE"),
            Self::False => write!(f, "FALSE"),
            Self::Pop => write!(f, "POP"),
            Self::DefineGlobal(index) => {
                write!(
                    f,
                    "DEFINE_GLOBAL {number:>width$}",
                    number = index,
                    width = 11
                )
            }
            Self::GetGlobal(index) => {
                write!(f, "GET_GLOBAL {number:>width$}", number = index, width = 14)
            }
            Self::SetGlobal(index) => {
                write!(f, "SET_GLOBAL {number:>width$}", number = index, width = 14)
            }
            Self::GetLocal(index) => {
                write!(f, "GET_LOCAL {number:>width$}", number = index, width = 15)
            }
            Self::SetLocal(index) => {
                write!(f, "SET_LOCAL {number:>width$}", number = index, width = 15)
            }
            Self::Equal => write!(f, "EQUAL"),
            Self::Greater => write!(f, "GREATER"),
            Self::Less => write!(f, "LESS"),
            Self::Add => write!(f, "ADD"),
            Self::Subtract => write!(f, "SUBTRACT"),
            Self::Multiply => write!(f, "MULTIPLY"),
            Self::Divide => write!(f, "DIVIDE"),
            Self::Not => write!(f, "NOT"),
            Self::Negate => write!(f, "NEGATE"),
            Self::Print => write!(f, "PRINT"),
            Self::JumpIfFalse(index) => {
                write!(f, "JUMP_IF_FALSE {number:>width$}", number = index, width = 11)
            }
            Self::Jump(index) => {
                write!(f, "JUMP {number:>width$}", number = index, width = 20)
            }
            Self::Return => write!(f, "RETURN"),
        }
    }
}
