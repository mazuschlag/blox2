use std::{
    env, fs,
    io::{self, BufRead, Write},
};

use crate::{chunk::*, compiler::*, value::*};

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

    pub fn repl(&mut self) -> RunResult {
        println!("=== Welcome to blox v2.0");
        println!("=== Enter 'q' or 'Q' to quit");
        print!("> ");
        io::stdout().flush().unwrap();
        for line in io::stdin().lock().lines() {
            let input = line.unwrap_or_else(|e| {
                eprintln!("Error reading input {e}");
                String::from("")
            });

            if input.is_empty() {
                print!("> ");
                io::stdout().flush().unwrap();
                continue;
            }

            if input.to_lowercase().trim() == "q" {
                println!("=== Goodbye!");
                return RunResult::Ok;
            }

            match self.interpret(input) {
                RunResult::CompileError(e) => eprintln!("Compile error: {e}"),
                RunResult::RuntimeError(e) => eprintln!("Compile error: {e}"),
                RunResult::Ok => (),
            }

            self.ip = 0;
            print!("> ");
            io::stdout().flush().unwrap();
        }

        RunResult::Ok
    }

    pub fn run_file(&mut self, path: &str) -> RunResult {
        match fs::read_to_string(path) {
            Ok(source) => self.interpret(source),
            Err(e) => RunResult::CompileError(format!("Failed to open file at {path}: {e}")),
        }
    }

    fn interpret(&mut self, source: String) -> RunResult {
        let compiler = Compiler::new(source);
        match compiler.compile() {
            Ok(chunk) => self.run(chunk),
            Err(e) => return RunResult::CompileError(e),
        }
    }

    fn run(&mut self, chunk: Chunk) -> RunResult {
        loop {
            let ip = self.ip;
            let op = chunk.read_op(ip);

            if env::var("DEBUG_TRACE_EXECUTION").is_ok_and(|var| var == "1") {
                chunk.disassemble_instruction(ip, op);
                self.stack_trace();
            }

            self.ip += 1;

            match *op {
                OpCode::Constant(index) => {
                    let constant = chunk.read_constant(index);
                    self.stack.push(*constant);
                }
                OpCode::Add => {
                    let add = |left, right| Value::Number(left + right);
                    if let Err(e) = self.binary_op(add) {
                        return RunResult::RuntimeError(e);
                    }
                }
                OpCode::Subtract => {
                    let sub = |left, right| Value::Number(left - right);
                    if let Err(e) = self.binary_op(sub) {
                        return RunResult::RuntimeError(e);
                    }
                }
                OpCode::Multiply => {
                    let mult = |left, right| Value::Number(left * right);
                    if let Err(e) = self.binary_op(mult) {
                        return RunResult::RuntimeError(e);
                    }
                }
                OpCode::Divide => {
                    let div = |left, right| Value::Number(left / right);
                    if let Err(e) = self.binary_op(div) {
                        return RunResult::RuntimeError(e);
                    }
                }
                OpCode::Negate => {
                    let value = self.stack_pop();
                    match value {
                        Value::Number(n) => self.stack.push(Value::Number(-n)),
                    }
                }
                OpCode::Return => {
                    let value = self.stack_pop();
                    println!("{}", value);
                    return RunResult::Ok;
                }
            }
        }
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
    where
        F: FnMut(f64, f64) -> Value,
    {
        match (self.stack_pop(), self.stack_pop()) {
            (Value::Number(right), Value::Number(left)) => {
                self.stack.push(op(left, right));
                Ok(())
            }
        }
    }

    fn stack_pop(&mut self) -> Value {
        self.stack
            .pop()
            .expect("Attempting to pop from stack when stack is empty")
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

pub enum RunResult {
    Ok,
    CompileError(String),
    RuntimeError(String),
}
