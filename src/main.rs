mod chunk;
mod common;
mod compiler;
mod scanner;
mod token;
mod vm;

use std::{
    env,
    process,
};

use vm::*;

const SUCCESS: i32 = 0;
const RUNTIME_ERROR: i32 = 1;
const COMPILE_ERROR: i32 = 2;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = Vm::new();

    let result = match args.len() {
        1 => vm.repl(),
        _ => vm.run_file(&args[1]),
    };
    
    match result {
        InterpretResult::RuntimeError(e) => {
            eprintln!("Runtime error: {e}");
            process::exit(RUNTIME_ERROR);
        }
        InterpretResult::CompileError(e) => {
            eprintln!("Compile error: {e}");
            process::exit(COMPILE_ERROR);
        }
        InterpretResult::Ok => process::exit(SUCCESS),
    }
}
