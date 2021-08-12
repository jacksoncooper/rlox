use std::error;
use std::process;

use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

pub enum LoxError {
    ScanError,
    ParseError
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

pub fn report(line: usize, location: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, location, message);
}

pub fn fatal<T, E: error::Error>(result: Result<T, E>, exit_code: i32) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            println!("fatal: {}", error.to_string());
            process::exit(exit_code);
        }
    }
}
