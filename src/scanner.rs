use crate::token::Token;
use crate::token_type::TokenType as TT;

struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner { source, tokens: Vec::new(), start: 0, current: 0, line: 1 }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token>{
        while !self.is_at_end() {
            self.start = self.current;
            // self.scan_token();
        }

        // TODO: It's sort of icky to stuff a null character into this token
        // because the EOF lexeme is not really a lexeme; it is not present in
        // the source code and has no associated text. The alternative is to
        // define the Token type as a lexeme with metadata _or_ EOF. I prefer
        // the simplicity of using the null character here so we don't have to
        // switch on a sum type.

        let end_of_file = Token::new(
            TT::EndOfFile,
            String::from("\0"),
            self.line
        );
        self.tokens.push(end_of_file);

        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
