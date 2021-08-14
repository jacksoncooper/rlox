use crate::error;
use crate::interpreter::object::Object;
use crate::parser::expression::Expr;
use crate::parser::statement::Stmt;
use crate::scanner::token::Token;
use crate::scanner::token_type::TokenType as TT;

pub mod expression;
pub mod statement;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    statements: Option<Vec<Stmt>>
}

struct Panic {
    token: Token,
    message: String
}

fn to_literal(token: Token) -> Object {
    match token.token_type {
        TT::Number(float)  => Object::Number(float),
        TT::String(string) => Object::String(string),
        TT::False          => Object::Boolean(false),
        TT::True           => Object::Boolean(true),
        TT::Nil            => Object::Nil,
        _                  => panic!("token does not contain a literal")
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0, statements: None }
    }

    pub fn parse(&mut self) {
        let mut had_error: bool = false;
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.is_at_end() {
            match self.statement() {
                Ok(statement) =>
                    statements.push(statement),
                Err(panic) => {
                    error::parse_error(&panic.token, &panic.message);
                    had_error = true;
                    self.synchronize();
                }
            }
        }

        if !had_error {
            self.statements = Some(statements);
        }
    }

    pub fn consume(self) -> Result<Vec<Stmt>, error::LoxError> {
        match self.statements {
            Some(statements) => Ok(statements),
            None => Err(error::LoxError::ParseError)
        }
    }

    fn statement(&mut self) -> Result<Stmt, Panic> {
        if self.advance_if(&[TT::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, Panic> {
        let value: Expr = self.expression()?;
        self.expect(TT::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, Panic> {
        let expr: Expr = self.expression()?;
        self.expect(TT::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> Result<Expr, Panic> {
        self.equality()
    }

    fn binary<O>(&mut self, operators: &[TT], operand: &O) -> Result<Expr, Panic>
        where O: Fn(&mut Self) -> Result<Expr, Panic>
    {
        // Parse a sequence of left-associative binary operators.
        
        let mut left: Expr = operand(self)?;

        while self.advance_if(operators) {
            let operator: Token = self.previous();
            let right: Expr = operand(self)?;

            left = Expr::Binary {
                left: Box::new(left),
                operator: operator,
                right: Box::new(right)
            }
        }

        Ok(left)
    }

    fn equality(&mut self) -> Result<Expr, Panic> {
        let operators = [TT::BangEqual, TT::EqualEqual];
        self.binary(&operators, &Parser::comparison)
    }

    fn comparison(&mut self) -> Result<Expr, Panic> {
        let operators = [TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual];
        self.binary(&operators, &Parser::term)
    }

    fn term(&mut self) -> Result<Expr, Panic> {
        let operators = [TT::Minus, TT::Plus];
        self.binary(&operators, &Parser::factor)
    }

    fn factor(&mut self) -> Result<Expr, Panic> {
        let operators = [TT::Slash, TT::Star];
        self.binary(&operators, &Parser::unary)
    }

    fn unary(&mut self) -> Result<Expr, Panic> {
        // Parse a sequence of right-associative unary operators. If the final
        // primary expression panics, the whole unary expression panics.

        let operators = [TT::Bang, TT::Minus];

        if self.advance_if(&operators) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;

            return Ok(Expr::Unary {
                operator: operator,
                right: Box::new(right)
            })
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Panic> {
        // We're not using advance_if() here, which the book calls match(),
        // because some of the inhabitants of TokenType carry a literal value
        // and testing for equality requires the construction of an arbitrary
        // literal for TT::Number and TT::String.

        let token_type: TT = self.peek().token_type;

        // TODO: Identifiers?

        if let TT::Number(_) | TT::String(_) | TT::False | TT::True | TT::Nil = token_type {
            let token: Token = self.advance();
            return Ok(Expr::Literal { value: to_literal(token) });
        }

        if token_type == TT::LeftParen {
            self.advance();
            let group: Expr = self.expression()?;
            self.expect(TT::RightParen, "Expect ')' after expression.")?;
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

    fn expect(&mut self, token_type: TT, message: &str) -> Result<(), Panic> {
        if self.check(TT::clone(&token_type)) {
            self.advance();
            return Ok(())
        }

        Err(self.panic_here(message))
    }

    fn panic_here(&self, message: &str) -> Panic {
        Panic {
            token: Token::clone(&self.peek()),
            message: message.to_string()
        }
    }

    fn synchronize(&mut self) {
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
