use crate::error;
use crate::token::Token;
use crate::token_type::TokenType as TT;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    identifier_key: usize,
    stumbled: bool,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.chars().collect(), // [1]
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            identifier_key: 0,
            stumbled: false,
        }
    }

    pub fn scan_tokens(&mut self) {
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
    }

    pub fn consume(self) -> Result<Vec<Token>, error::LoxError> {
        if self.stumbled {
            Err(error::LoxError::Scan)
        } else {
            Ok(self.tokens)
        }
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

            d if is_digit(d) => self.number(),
            c if is_alpha(c) => self.identifier(),

            _  => {
                // These characters will be ignored and not passed to the parser.
                error::scanner_error(self.line, "Unexpected character.");
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
        self.source[self.current + 1]
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
            error::scanner_error(self.line, "Unterminated string.");
            self.stumbled = true;
            return;
        }

        self.advance();

        let string = self.collect_lexeme(self.start + 1, self.current - 1);
        self.add_token(TT::String(string));
    }

    fn number(&mut self) {
        while is_digit(self.peek()) { self.advance(); }
       
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();
        }
        
        while is_digit(self.peek()) { self.advance(); }

        let lexeme = self.collect_lexeme(self.start, self.current);
        let maybe_number: Result<f64, _> = lexeme.parse();
        
        match maybe_number {
            Ok(number) => self.add_token(TT::Number(number)),
            Err(_) => {
                error::scanner_error(
                    self.line,
                    "Number cannot be represented with 64 bits."
                );
                self.stumbled = true;
            }
        }
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let identifier = self.collect_lexeme(self.start, self.current);

        // TODO: This is slow! Replace with HashMap when you can figure how
        // to allocate it statically.

        let token = match identifier.as_str() {
            "and"    => TT::And,
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
            "super"  => TT::Super(self.new_key()),
            "this"   => TT::This(self.new_key()),
            "true"   => TT::True,
            "var"    => TT::Var,
            "while"  => TT::While,
            _        => TT::Identifier(self.new_key(), identifier),
        };

        self.add_token(token);
    }

    fn new_key(&mut self) -> usize {
        let key = self.identifier_key;
        self.identifier_key += 1;
        key
    }

    fn collect_lexeme(&self, start: usize, end: usize) -> String {
        // Convert from a list of Unicode Scalar Values to a UTF-8 string.
        let substring: &[char] = &self.source[start..end];
        let lexeme: String = substring.iter().collect();
        lexeme
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_and_peek() {
        let mut scanner = Scanner::new("eggs");

        // Start state is correct.

        assert_eq!(scanner.start, 0);
        assert_eq!(scanner.current, 0);
        assert!(!scanner.is_at_end());

        assert_eq!(scanner.peek(), 'e');
        assert_eq!(scanner.peek_next(), 'g');

        // Peeking does not affect state.

        assert_eq!(scanner.start, 0);
        assert_eq!(scanner.current, 0);
        assert!(!scanner.is_at_end());

        // Consume and stop at second to last character.

        assert_eq!(scanner.advance(), 'e');
        assert_eq!(scanner.advance(), 'g');

        assert_eq!(scanner.peek(), 'g');
        assert_eq!(scanner.peek_next(), 's');

        // Stop at last character. Not yet off end.

        assert_eq!(scanner.advance(), 'g');

        assert_eq!(scanner.start, 0);
        assert_eq!(scanner.current, 3);
        assert!(!scanner.is_at_end());

        assert_eq!(scanner.peek(), 's');
        assert_eq!(scanner.peek_next(), '\0');

        // Now off end.

        assert_eq!(scanner.advance(), 's');

        assert_eq!(scanner.start, 0);
        assert_eq!(scanner.current, 4);
        assert!(scanner.is_at_end());

        assert_eq!(scanner.peek(), '\0');
        assert_eq!(scanner.peek_next(), '\0');
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
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
