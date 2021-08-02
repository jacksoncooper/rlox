mod ast_printer;
mod error;
mod expression;
mod lox;
mod scanner;
mod token;
mod token_type;

fn main() {
    lox::interact();
}
