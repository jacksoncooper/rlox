use crate::error;
use crate::expression::Expr;
use crate::token::Token;
use crate::token_type::TokenType as TT;

use self::Parse::{Success, Panic};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

// TODO: Should probably be using Result<Expr, ()> for the power of the
// question mark (?) operator. But Err(()) is not particularly readable in
// place of Panic.

enum Parse {
    Success(Expr),
    Panic
}

impl Parse {
    fn unwrap(self) -> Expr {
        match self {
            Success(expr) => expr,
            Panic => panic!("called `Parse::unwrap()` on a `Panic` value")
        }
    }
}

impl Parser {
    pub fn new(&self) -> Parser {
        Parser { tokens: Vec::new(), current: 0 }
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

        let left: Parse = operand(self);

        if let Panic = left {
            return Panic;
        }

        let mut left: Expr = left.unwrap();

        while self.discard(operators) {
            let operator: Token = self.previous();
            let right: Parse = operand(self);

            match right {
                Success(expr) =>
                    left = Expr::Binary {
                        left: Box::new(left),
                        operator: operator,
                        right: Box::new(expr)
                    },
                Panic => return Panic,
            }
        }

        Success(left)
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

        if self.discard(&operators) {
            let operator: Token = self.previous();
            let right: Parse = self.unary();

            return match right {
                Success(unary) => Success(Expr::Unary {
                    operator: operator,
                    right: Box::new(unary)
                }),
                Panic => Panic
            }
        }

        self.primary()
    }

    fn primary(&mut self) -> Parse {
        // We're not using discard() here, which the book calls match(),
        // because some of the inhabitants of TokenType carry a literal value
        // and testing for equality requires the construction of an arbitrary
        // literal for TT::Number and TT::String.

        let token_type: TT = self.peek().token_type;

        if let TT::Number(_) | TT::String(_) | TT::False | TT::True | TT::Nil = token_type {
            let token: Token = self.previous();
            return Success(Expr::Literal { value: token });
        }

        if token_type == TT::LeftParen {
            let expr: Parse = self.expression();

            match expr {
                Success(group) => {
                    if !self.discard(&[TT::RightParen]) {
                        error::parse_error(&self.peek(), "Expect ')' after expression.");
                        return Panic;
                    }

                    return Success(Expr::Grouping { grouping: Box::new(group) });
                },
                Panic => return Panic
            }
        }

        Panic
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

    fn discard(&mut self, token_types: &[TT]) -> bool {
        for token_type in token_types {
            if self.check(TT::clone(token_type)) {
                self.advance();
                return true;
            }
        }
        
        false
    }
}
