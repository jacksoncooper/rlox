use std::env;
use std::error;
use std::fs;
use std::io::{self, Write};
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
    fatal(fs::read_to_string(path), 66);
}

fn run_prompt() {
    loop {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        print!("> ");
        fatal(stdout.flush(), 74);

        let mut line = String::new();
        fatal(stdin.read_line(&mut line), 74);
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        run(&line);
    }
}

fn run(source: &str) {
}

fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn report(line: u32, location: &str, message: &str) {
    println!("[line {}] error{}: {}", line, location, message);
}

fn fatal<T>(result: Result<T, impl error::Error>, exit_code: i32) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            println!("fatal: {}", error.to_string());
            process::exit(exit_code);
        }
    }
}
