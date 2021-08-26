use crate::token::Token;
use crate::token_type::TokenType as TT;

pub enum LoxError {
    Scan, Parse, Resolve, Interpret,
}

pub fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}

pub fn scanner_error(line: usize, message: &str) {
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
