use std::env;
use std::error;
use std::fs;
use std::io::{self, Write};
use std::process;

use crate::error::LoxError;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;

// Exit codes from FreeBSD's 'sysexits.h' header: https://bit.ly/36JtSK0.

pub fn interact() {
    process::exit(match lox() {
        Err(exit_code) => exit_code,
        Ok(())         => 0
    });
}

fn lox() -> Result<(), i32> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => run_prompt(),
        1 => run_file(&args[0]),
        _ => {
            println!("usage: jlox [script]");
            Err(64)
        }
    }
}

fn run_file(path: &str) -> Result<(), i32> {
    let contents = fatal(fs::read_to_string(path), 66)?;
    let status = run(&contents);

    match status {
        Err(LoxError::Scan)      => Err(65),
        Err(LoxError::Parse)     => Err(65),
        Err(LoxError::Resolve)   => Err(65),
        Err(LoxError::Interpret) => Err(70),
        Ok(())                   => Ok(()),
    }
}

fn run_prompt() -> Result<(), i32> {
    loop {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        print!("> ");
        fatal(stdout.flush(), 74)?;

        let mut line = String::new();
        fatal(stdin.read_line(&mut line), 74)?;
        let line = line.trim();

        if line.is_empty() {
            return Ok(());
        }

        // Absorb any error from the scanner, parser, or interpreter.
        let _: Result<(), LoxError> = run(line);
    }
}

fn run(source: &str) -> Result<(), LoxError> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    let tokens = scanner.consume()?;

    // for token in tokens.iter() {
    //     println!("{:?}", token);
    // }

    let mut parser = Parser::new(tokens);
    parser.parse();
    let statements = parser.consume()?;

    // for statement in &statements {
    //     println!("{:#?}", statement);
    // }

    let mut resolver = Resolver::new();
    resolver.resolve_statements(&statements);
    let resolutions = resolver.consume()?;

    let mut interpreter = Interpreter::new(resolutions);
    interpreter.interpret(statements)?;

    Ok(())
}

pub fn fatal<T, E: error::Error>(result: Result<T, E>, exit_code: i32) -> Result<T, i32> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            eprintln!("fatal: {}", error.to_string());
            Err(exit_code)
        }
    }
}
