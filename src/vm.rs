use std::{
    env,
    fs,
    io::{self, BufRead, Write},
};

use crate::{
    chunk::*,
    common::*,
    compiler::*,
};

pub struct Vm {
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn repl(&mut self) -> InterpretResult {
        println!("=== Welcome to blox v1.0");
        println!("=== Enter 'q' or 'Q' to quit");
        print!("> ");
        io::stdout().flush().unwrap();
        for line in io::stdin().lock().lines() {
            match line {
                Ok(input) => {
                    if input.to_lowercase().trim() == "q" {
                        println!("=== Goodbye!");
                        return InterpretResult::Ok
                    }

                    match self.interpret(input) {
                        InterpretResult::CompileError(e) => eprintln!("Compile error: {e}"),
                        InterpretResult::RuntimeError(e) => eprintln!("Compile error: {e}"),
                        InterpretResult::Ok => (),
                    }

                    print!("> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    return InterpretResult::RuntimeError(e.to_string());
                }
            }
        }

        InterpretResult::Ok
    }

    pub fn run_file(&mut self, path: &str) -> InterpretResult {
        match fs::read_to_string(path) {
            Ok(source) => self.interpret(source),
            Err(e) => InterpretResult::CompileError(format!("Failed to open file at {path}: {e}")), 
        }
    }

    fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(source);
        match compiler.compile() {
            Ok(chunk) => self.run(chunk),
            Err(e) => return InterpretResult::CompileError(e),
        }
    }

    fn run(&mut self, chunk: Chunk) -> InterpretResult {
        loop {
            let ip = self.ip;
            let op = chunk.read_op(ip);
            
            if env::var("DEBUG_TRACE_EXECUTION").is_ok() {
                chunk.disassemble_instruction(ip, &op);
                self.stack_trace();
            }

            self.ip += 1;

            match *op {
                OpCode::Constant(index) => {
                    let constant = chunk.read_constant(index);
                    self.stack.push(*constant);
                }
                OpCode::Add => {
                    if let Err(e) = self.binary_op(|right, left| Value::Number(left + right)) {
                        return InterpretResult::RuntimeError(e);
                    }
                },
                OpCode::Subtract => {
                    if let Err(e) = self.binary_op(|right, left| Value::Number(left - right)) {
                        return InterpretResult::RuntimeError(e);
                    }
                },
                OpCode::Multiply => {
                    if let Err(e) = self.binary_op(|right, left| Value::Number(left * right)) {
                        return InterpretResult::RuntimeError(e);
                    }
                },
                OpCode::Divide => {
                    if let Err(e) = self.binary_op(|right, left| Value::Number(left / right)) {
                        return InterpretResult::RuntimeError(e);
                    }
                },
                OpCode::Negate => {
                    let value = self.stack_pop();
                    match value {
                        Value::Number(n) => self.stack.push(Value::Number(-n)),
                        _ => return InterpretResult::RuntimeError(String::from("Cannot negate a non-number"))
                    }
                }
                OpCode::Return => {
                    let value = self.stack_pop();
                    println!("{}", value);
                    return InterpretResult::Ok;
                }
            }
        }
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
    where
        F: FnMut(f64, f64) -> Value,
    {
        let (left, right) = (self.stack_pop(), self.stack_pop());
        match (left, right) {
            (Value::Number(b), Value::Number(a)) => {
                self.stack.push(op(a, b));
                Ok(())
            }
            _ => Err(String::from("Operand must be a number")),
        }
    }

    fn stack_pop(&mut self) -> Value {
        assert!(self.stack.len() > 0, "Attempting to pop from stack when stack is empty");
        self.stack.pop().unwrap()
    }

    fn stack_trace(&self) {
        print!("           ");
        for index in 0..self.stack_top() {
            print!("[ {} ]", self.stack[index]);
        }

        println!();
    }

    fn stack_top(&self) -> usize {
        self.stack.len()
    }
}

pub enum InterpretResult {
    Ok,
    CompileError(String),
    RuntimeError(String),
}