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

    pub fn repl(&mut self) -> Interpret {
        println!("=== Welcome to blox v2.0");
        println!("=== Enter 'q' or 'Q' to quit");
        print!("> ");
        io::stdout().flush().expect("Error flushing stdout.");
        for line in io::stdin().lock().lines() {
            let input = line.unwrap_or_else(|e| {
                eprintln!("Error reading input {e}");
                String::from("")
            });

            if input.is_empty() {
                print!("> ");
                io::stdout().flush().expect("Error flushing stdout.");
                continue;
            }

            if input.to_lowercase().trim() == "q" {
                println!("=== Goodbye!");
                return Interpret::Ok;
            }

            match self.interpret(input) {
                Interpret::CompileError(e) => eprintln!("Compile error: {e}"),
                _ => (),
            }

            self.reset_stack();
            print!("> ");
            io::stdout().flush().expect("Error flushing stdout.");
        }

        Interpret::Ok
    }

    pub fn run_file(&mut self, path: &str) -> Interpret {
        match fs::read_to_string(path) {
            Ok(source) => self.interpret(source),
            Err(e) => Interpret::CompileError(format!("Failed to open file at {path}: {e}")),
        }
    }

    fn interpret(&mut self, source: String) -> Interpret {
        let compiler = Compiler::new(source);
        match compiler.compile() {
            Ok(chunk) => self.run(chunk),
            Err(e) => return Interpret::CompileError(e),
        }
    }

    fn run(&mut self, chunk: Chunk) -> Interpret {
        loop {
            let ip = self.ip;
            let op = chunk.read_op(ip).to_owned();

            if env::var("DEBUG_TRACE_EXECUTION").is_ok_and(|var| var == "1") {
                chunk.disassemble_instruction(ip, &op);
                self.stack_trace();
            }

            self.ip += 1;

            match op {
                OpCode::Constant(index) => {
                    let constant = chunk.read_constant(index);
                    self.stack.push(*constant);
                }
                OpCode::Add => {
                    let add = |left, right| Value::Number(left + right);
                    if let Err(e) = self.binary_op(add) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                OpCode::Subtract => {
                    let sub = |left, right| Value::Number(left - right);
                    if let Err(e) = self.binary_op(sub) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                OpCode::Multiply => {
                    let mult = |left, right| Value::Number(left * right);
                    if let Err(e) = self.binary_op(mult) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                OpCode::Divide => {
                    let div = |left, right| Value::Number(left / right);
                    if let Err(e) = self.binary_op(div) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                       return self.runtime_error("Cannot negate a non-number.", &chunk);
                    }
        
                    if let Value::Number(n) = self.stack_pop() {
                        self.stack.push(Value::Number(-n));
                    }
                }
                OpCode::Return => {
                    let value = self.stack_pop();
                    println!("{}", value);
                    return Interpret::Ok;
                }
            }
        }
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
    where
        F: FnMut(f64, f64) -> Value,
    {
        if !self.peek(0).is_number() || !self.peek(1).is_number() {
            return Err(String::from("Cannot perform operation on two non-numbers."));
        }

        if let (Value::Number(right), Value::Number(left)) = (self.stack_pop(), self.stack_pop()) {
            self.stack.push(op(left, right));
        }

        Ok(())
    }

    fn stack_pop(&mut self) -> Value {
        self.stack
            .pop()
            .expect("Attempting to pop from stack when stack is empty")
    }

    fn peek(&self, distance: usize) -> &Value {
        let top = self.stack_top() - 1;
        self.stack.get(top - distance).expect("Failure to peek stack top")
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

    fn reset_stack(&mut self) {
        self.ip = 0;
        self.stack.clear();
    }

    fn runtime_error(&mut self, message: &str, chunk: &Chunk) -> Interpret {
        eprintln!("{message}");
        let ip = self.ip - 1;
        let line = chunk.get_line(ip);
        eprintln!("[line {line}] in script.");
        self.reset_stack();
        Interpret::RuntimeError
    }
}

pub enum Interpret {
    Ok,
    CompileError(String),
    RuntimeError,
}
