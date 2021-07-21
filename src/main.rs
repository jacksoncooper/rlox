mod lox;
mod token;
mod token_type;

fn main() {
    let mut my_lox = lox::Lox::new();
    my_lox.interact();
}
