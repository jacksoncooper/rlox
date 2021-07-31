use crate::error;
use crate::token::Token;
use crate::token_type::TokenType as TT;

pub struct Scanner {
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

            '/' => self.slash(),
            '"' => self.string(),

            ' ' | '\t' => (), '\n' => self.line += 1,

            digit if digit.is_ascii_digit() => self.number(),
            character if character.is_alphabetic() => self.identifier(),

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

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { return '\0'; }
        return self.source[self.current + 1];
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
        let lexeme = self.collect_lexeme(self.start, self.current);
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

    fn slash(&mut self) {
        if self.advance_if('/') {
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else {
            self.add_token(TT::Slash);
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }

        if self.is_at_end() {
            error::error(self.line, "Unterminated string.");
            self.had_error = true;
            return;
        }

        self.advance();

        let string = self.collect_lexeme(self.start + 1, self.current - 1);
        self.add_token(TT::String(string));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() { self.advance(); }
       
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
        }
        
        while self.peek().is_ascii_digit() { self.advance(); }

        let lexeme = self.collect_lexeme(self.start, self.current);
        let maybe_number: Result<f64, _> = lexeme.parse();
        
        match maybe_number {
            Ok(number) => self.add_token(TT::Number(number)),
            Err(_) => {
                error::error(self.line, "Number cannot be represented with 64 bits.");
                self.had_error = true;
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let identifier = self.collect_lexeme(self.start, self.current);

        let token = match identifier.as_str() {
            "and" => TT::And,
            "class"  => TT::Class,
            "else"   => TT::Else,
            "false"  => TT::False,
            "for"    => TT::For,
            "fun"    => TT::Fun,
            "if"     => TT::If,
            "nil"    => TT::Nil,
            "or"     => TT::Or,
            "print"  => TT::Print,
            "return" => TT::Return,
            "super"  => TT::Super,
            "this"   => TT::This,
            "true"   => TT::True,
            "var"    => TT::Var,
            "while"  => TT::While,
            _        => TT::Identifier(identifier)
        };

        self.add_token(token);
    }

    fn collect_lexeme(&self, start: usize, end: usize) -> String {
        // Convert from a list of Unicode Scalar Values to a UTF-8 string.
        let substring: &[char] = &self.source[start..end];
        let lexeme: String = substring.iter().collect();
        lexeme
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tests_tk() {
    }
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
// Even though our grammar is a subset of Unicode, i.e. ASCII, characters made
// of multiple USVs will not cause problems if they follow keywords due to the
// scanner's maximal munch policy. For example: elsé (U+0065 followed by
// U+0301) is one identifier, not "else" followed by an acute accent (U+0301).
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
