mod chunk;
mod compiler;
mod scanner;
mod token;
mod value;
mod vm;

use std::{env, process::ExitCode};

use vm::*;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let mut vm = Vm::new();

    let result = match args.len() {
        1 => vm.repl(),
        _ => vm.run_file(&args[1]),
    };

    match result {
        Interpret::Ok => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}
