use std::env;
use std::error;
use std::fs;
use std::io::{self, Write};
use std::process;

// Exit codes from FreeBSD's 'sysexits.h' header: https://bit.ly/36JtSK0

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Lox {
        Lox { had_error: false }
    }

    pub fn interact(&mut self) {
        let args: Vec<String> = env::args().skip(1).collect();
        if args.len() > 1 {
            println!("usage: jlox [script]");
            process::exit(64);
        } else if args.len() == 1 {
            self.run_file(&args[0]);
        } else {
            self.run_prompt();
        }
    }

    fn run_file(&mut self, path: &str) {
        let contents = fatal(fs::read_to_string(path), 66);
        self.run(&contents);

        if self.had_error {
            process::exit(65);
        }
    }

    fn run_prompt(&mut self) {
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

            self.run(&line);

            self.had_error = false;
        }
    }

    fn run(&mut self, source: &str) {
    }

    fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: u32, location: &str, message: &str) {
        println!("[line {}] error{}: {}", line, location, message);
    }
}

fn fatal<T, E: error::Error>(result: Result<T, E>, exit_code: i32) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            println!("fatal: {}", error.to_string());
            process::exit(exit_code);
        }
    }
}
