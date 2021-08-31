use std::rc::Rc;

use crate::callable::definitions as def;
use crate::error;
use crate::object::Object;
use crate::expression::Expr;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token_type::TokenType as TT;

struct Error {
    token: Token,
    message: String,
}

impl Error {
    fn new(token: Token, message: String) -> Error {
        Error { token, message }
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

type Tokens = std::iter::Peekable<std::vec::IntoIter<Token>>;

pub struct Parser {
    tokens: Tokens,
    statements: Vec<Stmt>,
    stumbled: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter().peekable(),
            statements: Vec::new(),
            stumbled: false,
        }
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
            if let Some(declaration) = self.declaration() {
                self.statements.push(declaration);
            }
        }
    }

    pub fn consume(self) -> Result<Vec<Stmt>, error::LoxError> {
        if !self.stumbled {
            Ok(self.statements)
        } else {
            Err(error::LoxError::Parse)
        }
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.advance_if(&[TT::Class]).is_some() {
            self.class_declaration()
        } else if self.advance_if(&[TT::Fun]).is_some() {
            self.function("function").map(Stmt::Function)
        } else if self.advance_if(&[TT::Var]).is_some() {
            self.variable_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(declaration) => Some(declaration),
            Err(panic) => {
                self.stumbled = true;
                error::parse_error(&panic.token, &panic.message);
                self.synchronize();
                None
            }
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.expect_identifier("Expect class name.".to_string())?;
       
        let parent = if self.advance_if(&[TT::Less]).is_some() {
            Some(self.expect_identifier("Expect superclass name.".to_string())?)
        } else { None };

        self.expect(TT::LeftBrace, "Expect '{' before class body.".to_string())?;

        let mut methods = Vec::new();
        while !self.check(&TT::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.expect(TT::RightBrace, "Expect '}' after class body.".to_string())?;

        Ok(Stmt::Class(def::Class(Rc::new(name), parent.map(Rc::new), methods)))
    }

    fn function(&mut self, kind: &str) -> Result<def::Function, Error> {
        let name = self.expect_identifier(
            format!("Expect {} name.", kind)
        )?;

        self.expect(
            TT::LeftParen,
            format!("Expect '(' after {} name.", kind)
        )?;

        let parameters = self.parameters()?;

        self.expect(
            TT::LeftBrace,
            format!("Expect '{{' before {} body.", kind)
        )?;

        let body = self.block()?;

        Ok(def::Function(
           Rc::new(name),
           Rc::new(parameters),
           Rc::new(body),
        ))
    }

    fn parameters(&mut self) -> Result<Vec<Token>, Error> {
        let mut parameters = Vec::new();
        let mut too_many = false;
    
        if !self.check(&TT::RightParen) {
            parameters.push(self.parameter()?);

            while self.advance_if(&[TT::Comma]).is_some() {
                if !too_many && parameters.len() >= 255 {
                    too_many = true;
                    error::parse_error(
                        self.peek(),
                        "Can't have more than 255 parameters."
                    );
                    self.stumbled = true;
                }
                parameters.push(self.parameter()?);
            }
        }

        self.expect(
            TT::RightParen,
            "Expect ')' after parameters.".to_string(),
        )?;

        Ok(parameters)
    }

    fn parameter(&mut self) -> Result<Token, Error> {
        let parameter = self.expect_identifier(
            "Expect parameter name.".to_string()
        )?;

        Ok(parameter)
    }

    fn variable_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.expect_identifier(
            "Expect variable name.".to_string()
        )?;

        let initializer = if self.advance_if(&[TT::Equal]).is_some() {
            Some(self.expression()?)
        } else {
            None
        };
    
        self.expect(
            TT::Semicolon,
            "Expect ';' after variable declaration.".to_string(),
        )?;

        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.advance_if(&[TT::For]).is_some() {
            return self.for_statement();
        }

        if self.advance_if(&[TT::If]).is_some() {
            return self.if_statement();
        }

        if self.advance_if(&[TT::LeftBrace]).is_some() {
            return Ok(Stmt::Block(self.block()?));
        }

        if self.advance_if(&[TT::Print]).is_some() {
            return self.print_statement();
        }

        if let Some(keyword) = self.advance_if(&[TT::Return]) {
            return self.return_statement(keyword);
        }

        if self.advance_if(&[TT::While]).is_some() {
            return self.while_statement();
        }

        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.expect(TT::LeftParen, "Expect '(' after 'if'.".to_string())?;
        let condition = self.expression()?;
        self.expect(TT::RightParen, "Expect ')' after condition.".to_string())?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = if self.advance_if(&[TT::Else]).is_some() {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.expect(TT::LeftParen, "Expect '(' after 'for'.".to_string())?;

        let initializer: Option<Stmt> =
            if self.advance_if(&[TT::Semicolon]).is_some() {
                None
            } else if self.advance_if(&[TT::Var]).is_some() {
                Some(self.variable_declaration()?)
            } else {
                Some(self.expression_statement()?)
            };

        let condition: Option<Expr> =
            if self.check(&TT::Semicolon) { None }
            else { Some(self.expression()?) };

        self.expect(TT::Semicolon, "Expect ';' after loop condition.".to_string())?;

        let increment: Option<Expr> =
            if self.check(&TT::RightParen) { None }
            else { Some(self.expression()?) };

        self.expect(TT::RightParen, "Expect ')' after for clauses.".to_string())?;

        let mut body: Stmt = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        let condition: Expr = condition.unwrap_or(
            Expr::Literal(Object::Boolean(true))
        );

        body = Stmt::While(condition, Box::new(body));

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TT::RightBrace) && !self.is_at_end() {
            if let Some(declaration) = self.declaration() {
                statements.push(declaration);
            }
        }

        self.expect(TT::RightBrace, "Expect '}' after block.".to_string())?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value: Expr = self.expression()?;
        self.expect(TT::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self, keyword: Token) -> Result<Stmt, Error> {
        let value = if !self.check(&TT::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
    
        self.expect(TT::Semicolon, "Expect ';' after return value.".to_string())?;
        Ok(Stmt::Return(keyword, value))
    }

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.expect(TT::LeftParen, "Expect '(' after 'while'.".to_string())?;
        let condition = self.expression()?;
        self.expect(TT::RightParen, "Expect ')' after condition.".to_string())?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While(condition, body))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expression: Expr = self.expression()?;
        self.expect(TT::Semicolon, "Expect ';' after expression.".to_string())?;
        Ok(Stmt::Expression(expression))
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr: Expr = self.or()?;

        if let Some(equals) = self.advance_if(&[TT::Equal]) {
            let value: Expr = self.assignment()?;

            return match expr {
                Expr::Variable(name) =>
                    Ok(Expr::Assignment(name, Box::new(value))),
                Expr::Get(object, name) =>
                    Ok(Expr::Set(object, name, Box::new(value))),
                _ => {
                    error::parse_error(&equals, "Invalid assignment target.");
                    self.stumbled = true;
                    Ok(value) // [1]
                }
            };
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Or];
        self.binary(&operators, &Parser::and, &Expr::Logical)
    }

    fn and(&mut self) -> Result<Expr, Error> {
        let operators = [TT::And];
        self.binary(&operators, &Parser::equality, &Expr::Logical)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let operators = [TT::BangEqual, TT::EqualEqual];
        self.binary(&operators, &Parser::comparison, &Expr::Binary)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual];
        self.binary(&operators, &Parser::term, &Expr::Binary)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Minus, TT::Plus];
        self.binary(&operators, &Parser::factor, &Expr::Binary)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let operators = [TT::Slash, TT::Star];
        self.binary(&operators, &Parser::unary, &Expr::Binary)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        // Parse a sequence of right-associative unary operators. If the final
        // primary expression panics, the whole unary expression panics.

        let operators = [TT::Bang, TT::Minus];

        if let Some(operator) = self.advance_if(&operators) {
            let right: Expr = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;

        loop {
            if self.advance_if(&[TT::LeftParen]).is_some() {
                expr = self.finish_call(expr)?;
            } else if self.advance_if(&[TT::Dot]).is_some() {
                let name = self.expect_identifier(
                    "Expect property name after '.'.".to_string()
                )?;
                expr = Expr::Get(Box::new(expr), name);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut arguments = Vec::new();
        let mut too_many = false;

        if !self.check(&TT::RightParen) {
            arguments.push(self.expression()?);

            while self.advance_if(&[TT::Comma]).is_some() {
                if !too_many && arguments.len() >= 255 {
                    too_many = true;
                    error::parse_error(
                        self.peek(),
                        "Can't have more than 255 arguments."
                    );
                    self.stumbled = true;
                }
                arguments.push(self.expression()?);
            }
        }

        let paren = self.expect(
            TT::RightParen,
            "Expect ')' after arguments.".to_string()
        )?;

        let callable = Expr::Call(
            Box::new(callee),
            Token::clone(&paren),
            arguments
        );

        Ok(callable)
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let next = self.peek();

        if let TT::Identifier(..) = next.token_type {
            return Ok(Expr::Variable(self.advance()));
        }

        if let TT::False     | TT::True
            |  TT::Number(_) | TT::String(_)
            |  TT::Nil
            = next.token_type {
            return Ok(Expr::Literal(to_object(self.advance())));
        }

        if let TT::LeftParen = next.token_type {
            self.advance();
            let group: Expr = self.expression()?;
            self.expect(TT::RightParen, "Expect ')' after expression.".to_string())?;
            return Ok(Expr::Grouping(Box::new(group)));
        }

        if let TT::This(..) = next.token_type {
            return Ok(Expr::This(self.advance()));
        }

        if let TT::Super(..) = next.token_type {
            let keyword = self.advance();
            self.expect(TT::Dot, "Expect '.' after 'super'.".to_string())?;
            let method = self.expect_identifier("Expect superclass method name.".to_string())?;
            return Ok(Expr::Super(keyword, method));
        }

        Err(Error::new(
            Token::clone(next),
            "Expect expression.".to_string()
        ))
    }

    fn binary<E, O>(
        &mut self,
        operators: &[TT], operand: &O, expression: &E
    ) -> Result<Expr, Error>
        where
            O: Fn(&mut Self) -> Result<Expr, Error>,
            E: Fn(Box<Expr>, Token, Box<Expr>) -> Expr
    {
        // Parse a sequence of left-associative binary operators.

        let mut left: Expr = operand(self)?;

        while let Some(operator) = self.advance_if(operators) {
            let right: Expr = operand(self)?;
            left = expression(Box::new(left), operator, Box::new(right));
        }

        Ok(left)
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TT::EndOfFile
    }

    fn peek(&mut self) -> &Token {
        if let Some(next) = self.tokens.peek() {
            return next;
        }

        // If the scanner doesn't terminate the programmer's input with an
        // end-of-file Token this is an error in the interpreter.
        panic!("expect EOF token at end");
    }

    fn check(&mut self, token_type: &TT) -> bool {
        if self.is_at_end() { return false; }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> Token {
        if let Some(previous) = self.tokens.next() {
            return previous;
        }

        // See peek().
        panic!("expect EOF token at end");
    }

    fn advance_if(&mut self, token_types: &[TT]) -> Option<Token> {
        for token_type in token_types {
            if self.check(token_type) {
                return Some(self.advance());
            }
        }
        
        None
    }

    fn expect(&mut self, token_type: TT, message: String) -> Result<Token, Error> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(Error::new(
            Token::clone(self.peek()),
            message
        ))
    }

    fn expect_identifier(&mut self, message: String) -> Result<Token, Error> {
        let next = self.peek();

        if let TT::Identifier(..) = next.token_type {
            return Ok(self.advance());
        }

        Err(Error::new(
            Token::clone(next),
            message
        ))
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            // If the current Token is a semicolon, the next Token starts a new
            // statement.

            if self.peek().token_type == TT::Semicolon {
                self.advance();
                return;
            }

            // Otherwise the Token may be a keyword which marks the start of a
            // statement.

            // TODO: Bob's implementation advances over the token that caused
            // the panic and will not synch on the following keywords out of
            // the gate. Mine does. I can't advance on the EOF token without
            // exhausting the token iterator and causing a panic.

            if let TT::Class  | TT::For | TT::Fun   | TT::If | TT::Print
                |  TT::Return | TT::Var | TT::While
                = self.peek().token_type { return; }

            self.advance();
        }
    }
}

// [1]

// An invalid assignment target is a recoverable error! Don't panic! This
// implementation excises the bad target and replaces it with the assignment's
// well-formed right operand. I don't fully understand why we don't immediately
// synchronize. Each operand to each assignment is fully parsed on the way
// down. We get to report multiple invalid assignment targets on the way up,
// though. Bob's implementation kicks up the malformed assignment target but
// never evaluates the AST. Mine kicks up the malformed target's value and also
// doesn't evaluate the AST after the parser recovers from a stumble.
