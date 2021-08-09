use crate::expression::Expr;

pub fn show(expr: &Expr) -> String {
    match expr {
        Expr::Binary { left, operator, right } =>
            parenthesize(&operator.lexeme, &[&left, &right]),
        Expr::Grouping { grouping } =>
            parenthesize("group", &[&grouping]),
        Expr::Literal { value } =>
            value.token_type.to_string(),
        Expr::Unary { operator, right } =>
            parenthesize(&operator.lexeme, &[&right])
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    let mut readable = String::from("(");

    readable.push_str(name);

    for expr in exprs {
        readable.push(' ');
        readable.push_str(&show(expr));
    }

    readable.push(')');

    readable
}

#[cfg(test)]
mod tests {
    use crate::token::Token;
    use crate::token_type::TokenType as TT;

    use super::*;

    #[test]
    fn show_expressions() {
        let integer = Token::new(
            TT::Number(123 as f64), 
            String::from("123"), 1
        );

        let floating = Token::new(
            TT::Number(45.67),
            String::from("45.67"), 1
        );

        let left_operand = Expr::Unary {
            operator: Token::new(TT::Minus, String::from("-"), 1),
            right: Box::new(Expr::Literal { value: integer })
        };

        let operator = Token::new(TT::Star, String::from("*"), 1);

        let right_operand = Expr::Grouping {
            grouping: Box::new(Expr::Literal { value: floating })
        };

        let binary_expression = Expr::Binary { 
            left: Box::new(left_operand),
            operator: operator,
            right: Box::new(right_operand),
        };

        assert_eq!(show(&binary_expression), "(* (- 123) (group 45.67))");

        let string = Token::new(
            TT::Identifier(String::from("eggs")),
            String::from("eggs"), 1
        );

        let string_expression = Expr::Literal { value: string };

        assert_eq!(print(&string_expression), "\"eggs\"");
    }
}
