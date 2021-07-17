use std::env;
use std::fs;
use std::process;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 1 {
        println!("usage: jlox [script]");
        process::exit(64);
    } else if args.len() == 1 {
        run_file(&args[0]);
    } else {
        run_prompt();
    }
}

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(contents) => run(&contents),
        Err(error) => println!("fatal: {}", error.to_string())
    }
}

fn run_prompt() {
}

fn run(source: &str) {
}

fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn report(line: u32, location: &str, message: &str) {
    println!("[line {}] error{}: {}", line, location, message);
}
