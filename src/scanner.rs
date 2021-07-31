use crate::error;
use crate::token::Token;
use crate::token_type::TokenType as TT;

struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    had_error: bool,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.chars().collect(), // [1]
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            had_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Option<&Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        let end_of_file = Token::new(
            TT::EndOfFile,
            String::from("\0"), // [2]
            self.line
        );

        self.tokens.push(end_of_file);

        if self.had_error { None } else { Some(&self.tokens) }
    }

    fn scan_token(&mut self) {
        match self.advance() {
            '(' => self.add_token(TT::LeftParen),
            ')' => self.add_token(TT::RightParen),
            '{' => self.add_token(TT::LeftBrace),
            '}' => self.add_token(TT::RightBrace),
            ',' => self.add_token(TT::Comma),
            '.' => self.add_token(TT::Dot),
            '-' => self.add_token(TT::Minus),
            '+' => self.add_token(TT::Plus),
            ';' => self.add_token(TT::Semicolon),
            '*' => self.add_token(TT::Star),
            '!' => self.add_token_if('=', TT::BangEqual, TT::Bang),
            '=' => self.add_token_if('=', TT::EqualEqual, TT::Equal),
            '<' => self.add_token_if('=', TT::LessEqual, TT::Less),
            '>' => self.add_token_if('=', TT::GreaterEqual, TT::Greater),
            '/' =>
                if self.advance_if('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TT::Slash);
                },
            ' ' | '\t' => (), '\n' => self.line += 1,
             _  => {
                // TODO: Report the column. Grapheme Clusters will complicate.
                error::error(self.line, "Unexpected character.");
                self.had_error = true;
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() { return '\0'; }
        self.source[self.current]
    }

    fn advance(&mut self) -> char {
        let current = self.current;
        self.current += 1;
        self.source[current]
    }

    fn advance_if(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        let next: char = self.source[self.current];
        if next != expected { return false; }
        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TT) {
        // Convert from a list of Unicode Scalar Values to a UTF-8 string.
        let substring: &[char] = &self.source[self.start..self.current];
        let lexeme: String = substring.iter().collect();
        let new_token = Token::new(token_type, lexeme, self.line);
        self.tokens.push(new_token);
    }

    fn add_token_if(&mut self, expected: char, success: TT, failure: TT) {
        if self.advance_if(expected) {
            self.add_token(success);
        } else {
            self.add_token(failure);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}

// [1]

// Collecting into Vec<char> is not idiomatic and is space inefficient, because
// we (1) copy the whole source to memory a second time and (2) Rust's char
// primitive uses four bytes while UTF-8 is a variable-width standard. The
// largest Unicode Scalar Value (USV) is 10FFFF_16 which is ~0.026 percent of
// the largest number representable with four bytes and ~1700 percent if a char
// were two bytes.
//
// Taking this excess memory associates each USV with a subscript, which is how
// the book implements the scanner, with the significant caveat that Java
// strings are encoded in UTF-16 so you're less likely to run into Characters
// that span multiple Code Points unless you're dealing with surrogates.
// Because our grammar is a subset of Unicode, i.e. ASCII, characters made of
// multiple USVs will cause problems if they follow keywords.
//
// We should really be working with grapheme clusters but I'd like to only use
// the standard library for this project. From the std::string::String
// documentation:
//
//   Iteration over grapheme clusters may be what you actually want. This
//   functionality is not provided by Rust’s standard library, check crates.io
//   instead.
//
// TLDR; Text encoding was a nonissue for me until Rust made me thing about it.

// [2]

// It's sort of icky to stuff a null character into this token because the EOF
// lexeme is not really a lexeme; it is not present in the source code and has
// no associated text. The alternative is to define the Token type as a lexeme
// with metadata _or_ EOF. I prefer the simplicity of using the null character
// here so we don't have to switch on a sum type.
