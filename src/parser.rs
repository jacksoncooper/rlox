use crate::error;
use crate::object::Object;
use crate::expression::Expr;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token_type::TokenType as TT;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    statements: Option<Vec<Stmt>>
}

struct Error {
    token: Token,
    message: String,
    recoverable: Option<Expr>,
}

impl Error {
    fn new(token: Token, message: &str, recoverable: Option<Expr>) -> Error {
        Error {
            token,
            message: message.to_string(),
            recoverable,
        }
    }

    fn recover(self) -> Result<Expr, Error> {
        match self.recoverable {
            Some(expr) => {
                error::parse_error(&self.token, &self.message);
                Ok(expr)
            },
            None => Err(self),
        }
    }
}

fn to_object(token: Token) -> Object {
    match token.token_type {
        TT::False          => Object::Boolean(false),
        TT::True           => Object::Boolean(true),
        TT::Number(float)  => Object::Number(float),
        TT::String(string) => Object::String(string),
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
            match self.declaration() {
                Ok(declaration) =>
                    statements.push(declaration),
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
            None => Err(error::LoxError::Parse)
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.advance_if(&[TT::Var]) {
            return self.variable_declaration();
        }

        self.statement()
    }

    fn variable_declaration(&mut self) -> Result<Stmt, Error> {
        let name: Token = self.peek();

        match name.token_type {
            TT::Identifier(_) => {
                self.advance();
            }
            _ => return Err(self.panic_here("Expect variable name.")),
        }

        let initializer = if self.advance_if(&[TT::Equal]) {
            Some(self.expression()?)
        } else { None };
    
        self.expect(TT::Semicolon, "Expect ';' after variable declaration.")?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.advance_if(&[TT::If]) {
            return self.if_statement();
        }

        if self.advance_if(&[TT::LeftBrace]) {
            return Ok(Stmt::Block { statements: self.block()? });
        }

        if self.advance_if(&[TT::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.expect(TT::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.expect(TT::RightParen, "Expect ')' after condition.")?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = if self.advance_if(&[TT::Else]) {
            Some(Box::new(self.statement()?))
        } else { None };

        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TT::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.expect(TT::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value: Expr = self.expression()?;
        self.expect(TT::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr: Expr = self.expression()?;
        self.expect(TT::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr: Expr = self.equality()?;

        if self.advance_if(&[TT::Equal]) {
            let equals: Token = self.previous();

            let value: Expr = match self.assignment() {
                Ok(value) => value,
                Err(error) => error.recover()?,
            };

            return match expr {
                Expr::Variable { name } =>
                    Ok(Expr::Assignment { name, value: Box::new(value) }), // [1]
                _ =>
                    Err(Error::new(
                        equals,
                        "Invalid assignment target.",
                        Some(value)
                    )),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let operators = [TT::BangEqual, TT::EqualEqual];
        self.binary(&operators, &Parser::comparison)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual];
        self.binary(&operators, &Parser::term)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Minus, TT::Plus];
        self.binary(&operators, &Parser::factor)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Slash, TT::Star];
        self.binary(&operators, &Parser::unary)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        // Parse a sequence of right-associative unary operators. If the final
        // primary expression panics, the whole unary expression panics.

        let operators = [TT::Bang, TT::Minus];

        if self.advance_if(&operators) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right)
            })
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token: Token = self.peek();

        if let TT::Identifier(_) = token.token_type {
            self.advance();

            return Ok(Expr::Variable {
                name: token,
            });
        }

        if let TT::False     | TT::True
            |  TT::Number(_) | TT::String(_)
            |  TT::Nil
            = token.token_type {

            self.advance();

            return Ok(Expr::Literal {
                value: to_object(token),
            });
        }

        if let TT::LeftParen = token.token_type {
            self.advance();
            let group: Expr = self.expression()?;
            self.expect(TT::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping { grouping: Box::new(group) });
        }

        Err(self.panic_here("Expect expression."))
    }

    fn binary<O>(&mut self, operators: &[TT], operand: &O) -> Result<Expr, Error>
        where O: Fn(&mut Self) -> Result<Expr, Error>
    {
        // Parse a sequence of left-associative binary operators.
        
        let mut left: Expr = operand(self)?;

        while self.advance_if(operators) {
            let operator: Token = self.previous();
            let right: Expr = operand(self)?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right)
            }
        }

        Ok(left)
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TT::EndOfFile
    }

    // TODO: The functions peek() and previous() are indicative of a design
    // problem. Cloning a Token can be very expensive when that Token contains
    // a literal. A rewrite should allocate Tokens on the heap and wrap them in
    // a reference-counting type like Rc. The clone() is to prevent the parser
    // from mutating its state and invalidating the reference.

    fn peek(&self) -> Token {
        Token::clone(&self.tokens[self.current])
    }

    fn previous(&self) -> Token {
        Token::clone(&self.tokens[self.current - 1])
    }

    fn check(&self, token_type: &TT) -> bool {
        if self.is_at_end() { return false; }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        
        self.previous()
    }

    fn advance_if(&mut self, token_types: &[TT]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        
        false
    }

    fn expect(&mut self, token_type: TT, message: &str) -> Result<Token, Error> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(self.panic_here(message))
    }

    fn panic_here(&self, message: &str) -> Error {
        Error::new(self.peek(), message, None)
    }

    fn synchronize(&mut self) {
        // Discard the Token that caused the panic.
        self.advance();

        while !self.is_at_end() {
            // If the current Token is a semicolon, the next Token starts a new
            // statement.

            if self.previous().token_type == TT::Semicolon { return; }

            // Otherwise the Token may be a keyword which marks the start of a
            // statement.

            let token_type: TT = self.peek().token_type;

            // TODO: This is like advance_if() without the advance.

            if let TT::Class  | TT::For | TT::Fun   | TT::If | TT::Print
                |  TT::Return | TT::Var | TT::While
                = token_type { return; }

            self.advance();
        }
    }
}

// [1]

// An invalid assignment target is a recoverable error! Don't panic! TODO:
// Because Rust doesn't have exceptions, and I'm not using global mutable
// state, which I'm not even sure Rust supports, this Lox implementation
// excises the bad target and replaces it with its well-formed right operand. I
// don't fully understand why we don't immediately synchronize. Each operand to
// each assignment is fully parsed on the way down. We get to report multiple
// invalid assignment targets on the way up, though. Bob's implementation kicks
// up the malformed assignment target but never evaluates the AST. Mine does
// the opposite.
