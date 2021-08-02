use crate::expression::Expr;

fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary { left, operator, right } =>
            parenthesize(operator.lexeme(), vec![ &left, &right ]),
        Expr::Grouping { grouping } =>
            parenthesize("group", vec![ &grouping ]),
        Expr::Literal { value } =>
            value.to_string(),
        Expr::Unary { operator, right } =>
            parenthesize(operator.lexeme(), vec![ &right ])
    }
}

fn parenthesize(name: &str, exprs: Vec<&Expr>) -> String {
    let mut readable = String::from("(");

    readable.push_str(name);

    for expr in exprs {
        readable.push(' ');
        readable.push_str(&print(expr));
    }

    readable.push(')');

    readable
}

#[cfg(test)]
mod tests {
    use crate::token::Token;
    use crate::token_type::{self, TokenType as TT};

    use super::*;

    #[test]
    fn print_expression() {
        let number = TT::Number(123 as f64);
        let another_number = TT::Number(45.67);
        let eggs = TT::Identifier(String::from("eggs"));

        let literal = Expr::Literal {
            value: &token_type::to_literal(number).unwrap()
        };
        
        let another_literal =Expr::Literal {
            value: &token_type::to_literal(another_number).unwrap()
        };

        let left_operand = Expr::Unary {
            operator: &Token::new(TT::Minus, String::from("-"), 1),
            right: Box::new(literal)
        };

        let operator = Token::new(TT::Star, String::from("*"), 1);

        let right_operand = Expr::Grouping {
            grouping: Box::new(another_literal)
        };

        let binary_expression = Expr::Binary { 
            left: Box::new(left_operand),
            operator: &operator,
            right: Box::new(right_operand),
        };

        let string_expression = Expr::Literal {
            value: &token_type::to_literal(eggs).unwrap()
        };

        assert_eq!(print(&string_expression), "\"eggs\"");
        assert_eq!(print(&binary_expression), "(* (- 123) (group 45.67))");
    }
}
