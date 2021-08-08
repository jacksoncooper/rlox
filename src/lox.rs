use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use crate::ast_printer;
use crate::error;
use crate::parser::Parser;
use crate::scanner::Scanner;

// Exit codes from FreeBSD's 'sysexits.h' header: https://bit.ly/36JtSK0

pub fn interact() {
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
    let contents = error::fatal(fs::read_to_string(path), 66);
    let success = run(&contents);
    if success.is_err() {
        process::exit(65);
    }
}

fn run_prompt() {
    loop {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        print!("> ");
        error::fatal(stdout.flush(), 74);

        let mut line = String::new();
        error::fatal(stdin.read_line(&mut line), 74);
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        let _ = run(&line);
    }
}

fn run(source: &str) -> Result<(), error::LoxError> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    let tokens = scanner.consume()?;

    for token in tokens.iter() {
        println!("{:?}", token);
    }
    
    let mut parser = Parser::new(tokens);
    parser.parse();
    let expr = parser.consume()?;

    println!("{}", ast_printer::show(&expr));
    
    Ok(())
}
