use std::error;
use std::process;

use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

pub enum LoxError {
    ScanError,
    ParseError,
    InterpretError,
}

pub fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}

pub fn syntax_error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn parse_error(token: &Token, message: &str) {
    if token.token_type == TT::EndOfFile {
        report(token.line, " at end", message);
    } else {
        let location: String = format!(" at '{}'", token.lexeme);
        report(token.line, &location, message);
    }
}

pub fn runtime_error(token: &Token, message: &str) {
    eprintln!("{}\n[line {}]", message, token.line);
}

pub fn fatal<T, E: error::Error>(result: Result<T, E>, exit_code: i32) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("fatal: {}", error.to_string());
            process::exit(exit_code);
        }
    }
}
