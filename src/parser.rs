use crate::expression::Expr;
use crate::token::Token;
use crate::token_type::TokenType as TT;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

impl Parser {
    pub fn new(&self) -> Parser {
        Parser { tokens: Vec::new(), current: 0 }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn binary<O>(&mut self, operators: &[TT], operand: &O) -> Expr
        where O: Fn(&mut Self) -> Expr
    {
        // Parse a sequence of left-associative binary operators.

        let mut expr: Expr = operand(&mut self);

        while self.discard(operators) {
            let operator: &Token = self.previous();
            let right: Expr = operand(&mut self);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: Token::clone(operator),
                right: Box::new(right)
            };
        }

        expr
    }

    fn equality(&mut self) -> Expr {
        let operators = [TT::BangEqual, TT::EqualEqual];
        self.binary(&operators, &Parser::comparison)
    }

    fn comparison(&mut self) -> Expr {
        let operators = [TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual];
        self.binary(&operators, &Parser::term)
    }

    fn term(&mut self) -> Expr {
        let operators = [TT::Minus, TT::Plus];
        self.binary(&operators, &Parser::factor)
    }

    fn factor(&mut self) -> Expr {
        let operators = [TT::Slash, TT::Star];
        self.binary(&operators, &Parser::unary)
    }

    fn unary(&mut self) -> Expr {
        // Parse a sequence of right-associative unary operators.

        let operators = [TT::Bang, TT::Minus];

        if self.discard(&operators) {
            let operator: &Token = self.previous();
            let right: Expr = self.unary();

            return Expr::Unary {
                operator: Token::clone(operator),
                right: Box::new(right)
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        let token_type: TT = self.peek().token_type;

        if let TT::Number(_) | TT::String(_) | TT::False | TT::True | TT::Nil = token_type {
            let token: &Token = self.previous();
            return Expr::Literal { value: Token::clone(token) };
        }

        if self.discard(&[TT::LeftParen]) {
            let expr: Expr = self.expression();
            self.consume(TT::RightParen, "Expect ')' after expression.");
            return Expr::Grouping { grouping: Box::new(expr) };
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        &self.previous()
    }

    fn discard(&mut self, token_types: &[TT]) -> bool {
        for &token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        
        false
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TT::EndOfFile
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn check(&self, token_type: TT) -> bool {
        if self.is_at_end() { return false; }
        return self.peek().token_type == token_type;
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
