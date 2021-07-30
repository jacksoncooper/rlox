mod lox;
mod scanner;
mod token;
mod token_type;

use lox::Lox;

fn main() {
    let mut my_lox = Lox::new();
    my_lox.interact();
}
