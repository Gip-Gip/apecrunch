use crate::parser::Token;

// op_engine::get_equality - get the equality of an expression by recursively simplifying it
//
// ARGUMENTS:
//  token: &Token - tokenized expression to get the equality of
//
// DESCRIPTION:
//  This function gets the equality of an expression by recursively simplifying it
pub fn get_equality(tokens: &Token) -> Token
{
    return Token::Equality
    (
        Box::new(tokens.clone()),
        Box::new(simplify(tokens))
    );
}

// op_engine::simplify - recursively simplify an expression
//
// ARGUMENTS:
//  token: &Token - tokenized expression to simplify
//
// DESCRIPTION:
//  This function recursively simplifies an expression
pub fn simplify(tokens: &Token) -> Token
{
    return match tokens
    {
        // Almost all of these match cases are the same, understand this one and you understand them all...
        Token::Multiply(left, right) =>
        {
            let left_result = simplify(left); // Recursively simplify the left side
            let right_result = simplify(right); // Recursively simplify the right side

            // If both sides are numbers, operate on them and return a number token
            if let Token::Number(left_number) = &left_result
            {
                if let Token::Number(right_number) = &right_result
                {
                    return Token::Number(left_number.multiply(&right_number)); // In this case we multiply the two
                }
            }

            // Otherwise it cannot be further simplified, and we must return a multiply token
            return Token::Multiply(Box::new(left_result), Box::new(right_result));
        }

        Token::Divide(left, right) =>
        {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result
            {
                if let Token::Number(right_number) = &right_result
                {
                    return Token::Number(left_number.divide(&right_number));
                }
            }
            
            return Token::Divide(Box::new(left_result), Box::new(right_result));
        }

        Token::Add(left, right) =>
        {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result
            {
                if let Token::Number(right_number) = &right_result
                {
                    return Token::Number(left_number.add(&right_number));
                }
            }
            
            return Token::Add(Box::new(left_result), Box::new(right_result));
        }

        Token::Subtract(left, right) =>
        {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result
            {
                if let Token::Number(right_number) = &right_result
                {
                    return Token::Number(left_number.subtract(&right_number));
                }
            }
            
            return Token::Subtract(Box::new(left_result), Box::new(right_result));
        }

        Token::Equality(left, right) =>
        {
            let left_result = simplify(left);
            let right_result = simplify(right);

            return Token::Boolean(left_result == right_result);
        }

        Token::Number(number) =>
        {
            Token::Number(number.clone())
        }
        
        // It is entirely possible I am still a terrible programmer and somehow I haven't implemented all of the tokens...
        _ =>
        {
            panic!("Fatal Oopsiedaisies!\n\n\tExpression parsed but the op-engine is not able to simplify on it! {}", tokens.to_string());
        }
    }
}