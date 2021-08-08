use crate::error;
use crate::expression::Expr;
use crate::token::Token;
use crate::token_type::TokenType as TT;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    result: Option<Expr>
}

struct Panic {
    token: Token,
    message: String
}

type Parse = Result<Expr, Panic>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0, result: None }
    }

    pub fn parse(&mut self) {
        self.result = match self.expression() {
            Ok(expr) => Some(expr),
            Err(panic) => {
                error::parse_error(&panic.token, &panic.message);
                None
            }
        };
    }

    pub fn consume(self) -> Result<Expr, error::LoxError> {
        match self.result {
            Some(expr) => Ok(expr),
            None => Err(error::LoxError::ParseError)
        }
    }

    fn expression(&mut self) -> Parse {
        self.equality()
    }

    fn binary<O>(&mut self, operators: &[TT], operand: &O) -> Parse
        where O: Fn(&mut Self) -> Parse
    {
        // Parse a sequence of left-associative binary operators.
        
        // TODO: This is really some horrific Rust, caused by trying to capture
        // failure in the Parse type.

        let mut left: Expr = operand(self)?;

        while self.advance_if(operators) {
            let operator: Token = self.previous();
            let right: Parse = operand(self);

            match right {
                Ok(expr) =>
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: operator,
                        right: Box::new(expr)
                    },
                error => return error,
            }
        }

        Ok(left)
    }

    fn equality(&mut self) -> Parse {
        let operators = [TT::BangEqual, TT::EqualEqual];
        self.binary(&operators, &Parser::comparison)
    }

    fn comparison(&mut self) -> Parse {
        let operators = [TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual];
        self.binary(&operators, &Parser::term)
    }

    fn term(&mut self) -> Parse {
        let operators = [TT::Minus, TT::Plus];
        self.binary(&operators, &Parser::factor)
    }

    fn factor(&mut self) -> Parse {
        let operators = [TT::Slash, TT::Star];
        self.binary(&operators, &Parser::unary)
    }

    fn unary(&mut self) -> Parse {
        // Parse a sequence of right-associative unary operators. If the final
        // primary expression panics, the whole unary expression panics.

        // TODO: Maybe rewrite this in the iterative style used for binary()?

        let operators = [TT::Bang, TT::Minus];

        if self.advance_if(&operators) {
            let operator: Token = self.previous();
            let right: Parse = self.unary();

            return match right {
                Ok(unary) => Ok(Expr::Unary {
                    operator: operator,
                    right: Box::new(unary)
                }),
                error => error
            }
        }

        self.primary()
    }

    fn primary(&mut self) -> Parse {
        // We're not using advance_if() here, which the book calls match(),
        // because some of the inhabitants of TokenType carry a literal value
        // and testing for equality requires the construction of an arbitrary
        // literal for TT::Number and TT::String.

        let token_type: TT = self.peek().token_type;

        // TODO: Identifiers?

        if let TT::Number(_) | TT::String(_) | TT::False | TT::True | TT::Nil = token_type {
            let token: Token = self.advance();
            return Ok(Expr::Literal { value: token });
        }

        if token_type == TT::LeftParen {
            self.advance();

            let group: Expr = self.expression()?;

            if !self.advance_if(&[TT::RightParen]) {
                return Err(self.panic_here("Expect ')' after expression."));
            }

            return Ok(Expr::Grouping { grouping: Box::new(group) });
        }

        Err(self.panic_here("Expect expression."))
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TT::EndOfFile
    }

    fn peek(&self) -> Token {
        Token::clone(&self.tokens[self.current])
    }

    fn previous(&self) -> Token {
        Token::clone(&self.tokens[self.current - 1])
    }

    fn check(&self, token_type: TT) -> bool {
        if self.is_at_end() { return false; }
        return self.peek().token_type == token_type;
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous()
    }

    fn advance_if(&mut self, token_types: &[TT]) -> bool {
        for token_type in token_types {
            if self.check(TT::clone(token_type)) {
                self.advance();
                return true;
            }
        }
        
        false
    }

    fn panic_here(&self, message: &str) -> Panic {
        Panic {
            token: Token::clone(&self.peek()),
            message: message.to_string()
        }
    }

    fn synchronize(&mut self) {
        // The current Token violates the rule we're processing. Discard it.
        self.advance();

        while !self.is_at_end() {
            // If the current Token is a semicolon, the next Token starts a new
            // statement.

            if self.advance_if(&[TT::Semicolon]) {
                return;
            }

            // Otherwise the Token may be a keyword which marks the start of a
            // statement.

            let token_type: TT = self.peek().token_type;

            // TODO: This is like advance_if() without the advance.

            if let TT::Class | TT::For | TT::Fun | TT::If | TT::Print
                | TT::Return | TT::Var | TT::While = token_type {
                return;
            }

            self.advance();
        }
    }
}
