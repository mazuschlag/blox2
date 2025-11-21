use std::{
    borrow::Borrow,
    env, fs,
    io::{self, BufRead, Write},
    rc::Rc,
};

use crate::{chunk::*, compiler::*, value::*};

#[derive(Debug, Clone)]
pub struct Vm {
    ip: usize,
    stack: Vec<Value>,
    objects: Rc<Obj>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            objects: Rc::new(Obj::Unit),
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

            self.interpret(input);
            self.reset_stack();
            print!("> ");
            io::stdout().flush().expect("Error flushing stdout.");
        }

        Interpret::Ok
    }

    pub fn run_file(&mut self, path: &str) -> Interpret {
        match fs::read_to_string(path) {
            Ok(source) => self.interpret(source),
            Err(e) => {
                eprintln!("Failed to open file at {path}: {e}");
                Interpret::RuntimeError
            }
        }
    }

    fn interpret(&mut self, source: String) -> Interpret {
        let compiler = Compiler::new(source, Rc::clone(&self.objects));
        match compiler.compile() {
            Ok(compiled) => {
                self.objects = compiled.objects;
                self.run(compiled.chunk)
            }
            Err(()) => return Interpret::CompileError,
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
                Op::Constant(index) => self.push(chunk.read_constant(index).clone()),
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Pop => _ = self.pop(),
                Op::Equal => {
                    let (second, first) = (self.pop(), self.pop());
                    self.push(Value::Bool(first == second));
                }
                Op::Greater => {
                    if let Err(e) = self.binary_op(|left, right| Value::Bool(left > right)) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                Op::Less => {
                    if let Err(e) = self.binary_op(|left, right| Value::Bool(left < right)) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                Op::Add => {
                    if let Err(e) = self.add() {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                Op::Subtract => {
                    if let Err(e) = self.binary_op(|left, right| Value::Number(left - right)) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                Op::Multiply => {
                    if let Err(e) = self.binary_op(|left, right| Value::Number(left * right)) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                Op::Divide => {
                    if let Err(e) = self.binary_op(|left, right| Value::Number(left / right)) {
                        return self.runtime_error(&e, &chunk);
                    }
                }
                Op::Not => {
                    let value = self.pop();
                    self.push(Value::Bool(value.is_falsey()));
                }
                Op::Negate => {
                    if !self.peek(0).is_number() {
                        return self.runtime_error("Cannot negate a non-number.", &chunk);
                    }

                    if let Value::Number(n) = self.pop() {
                        self.push(Value::Number(-n));
                    }
                }
                Op::Print => println!("{}", self.pop()),
                Op::Return => return Interpret::Ok,
            }
        }
    }

    fn add(&mut self) -> Result<(), String> {
        match (self.peek(0), self.peek(1)) {
            (Value::Obj(b), Value::Obj(a)) => self.concatenate(Rc::clone(a), Rc::clone(b)),
            (Value::Number(_), Value::Number(_)) => {
                self.binary_op(|left, right| Value::Number(left + right))
            }
            _ => Err(String::from("Operands must both be strings or numbers.")),
        }
    }

    fn concatenate(&mut self, a: Rc<Obj>, b: Rc<Obj>) -> Result<(), String> {
        match (a.borrow(), b.borrow()) {
            (Obj::Str(_, a), Obj::Str(_, b)) => {
                let string = Rc::new(Obj::Str(Rc::clone(&self.objects), format!("{a}{b}")));
                self.objects = Rc::clone(&string);
                let value = Value::Obj(Rc::clone(&string));
                self.push(value);
                Ok(())
            }
            _ => Err(String::from("Operands must both be strings.")),
        }
    }

    fn binary_op<F>(&mut self, mut op: F) -> Result<(), String>
    where
        F: FnMut(f64, f64) -> Value,
    {
        if !self.peek(0).is_number() || !self.peek(1).is_number() {
            return Err(String::from("Operands must both be numbers."));
        }

        if let (Value::Number(right), Value::Number(left)) = (self.pop(), self.pop()) {
            self.push(op(left, right));
        }

        Ok(())
    }

    fn pop(&mut self) -> Value {
        self.stack
            .pop()
            .expect("Attempting to pop from stack when stack is empty")
    }

    fn peek(&self, distance: usize) -> &Value {
        let top = self.stack_top() - 1;
        self.stack
            .get(top - distance)
            .expect("Stack peek index is out-of-bounds")
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
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
    CompileError,
    RuntimeError,
}
