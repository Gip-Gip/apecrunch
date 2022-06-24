// Copyright (c) 2022 Charles M. Thompson
//
// This file is part of ApeCrunch.
//
// ApeCrunch is free software: you can redistribute it and/or modify it under
// the terms only of version 3 of the GNU General Public License as published
// by the Free Software Foundation
//
// ApeCrunch is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
//
// You should have received a copy of the GNU General Public License along with
// ApeCrunch(in a file named COPYING).
// If not, see <https://www.gnu.org/licenses/>.

use crate::parser::Token;

// op_engine::get_equality - get the equality of an expression by recursively simplifying it
//
// ARGUMENTS:
//  token: &Token - tokenized expression to get the equality of
//
// DESCRIPTION:
//  This function gets the equality of an expression by recursively simplifying it
pub fn get_equality(tokens: &Token) -> Token {
    return Token::Equality(Box::new(tokens.clone()), Box::new(simplify(tokens)));
}

// op_engine::simplify - recursively simplify an expression
//
// ARGUMENTS:
//  token: &Token - tokenized expression to simplify
//
// DESCRIPTION:
//  This function recursively simplifies an expression
pub fn simplify(tokens: &Token) -> Token {
    return match tokens {
        // Almost all of these match cases are the same, understand this one and you understand them all...
        Token::Multiply(left, right) => {
            let left_result = simplify(left); // Recursively simplify the left side
            let right_result = simplify(right); // Recursively simplify the right side

            // If both sides are numbers, operate on them and return a number token
            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Token::Number(left_number.multiply(&right_number)); // In this case we multiply the two
                }
            }

            // Otherwise it cannot be further simplified, and we must return a multiply token
            return Token::Multiply(Box::new(left_result), Box::new(right_result));
        }

        Token::Divide(left, right) => {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Token::Number(left_number.divide(&right_number));
                }
            }

            return Token::Divide(Box::new(left_result), Box::new(right_result));
        }

        Token::Add(left, right) => {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Token::Number(left_number.add(&right_number));
                }
            }

            return Token::Add(Box::new(left_result), Box::new(right_result));
        }

        Token::Subtract(left, right) => {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Token::Number(left_number.subtract(&right_number));
                }
            }

            return Token::Subtract(Box::new(left_result), Box::new(right_result));
        }

        Token::Exponent(left, right) => {
            let left_result = simplify(left);
            let right_result = simplify(right);

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Token::Number(left_number.exponent(&right_number));
                }
            }

            return Token::Exponent(Box::new(left_result), Box::new(right_result));
        }

        Token::Equality(left, right) => {
            let left_result = simplify(left);
            let right_result = simplify(right);

            return Token::Boolean(left_result == right_result);
        }

        Token::Number(number) => Token::Number(number.clone()),

        // It is entirely possible I am still a terrible programmer and somehow I haven't implemented all of the tokens...
        _ => {
            panic!("Fatal Oopsiedaisies!\n\n\tExpression parsed but the op-engine is not able to simplify on it! {}", tokens.to_string());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    const TWO: &str = "2";
    const TWOPTWO: &str = "2 + 2";
    const TWOSTWO: &str = "2 - 2";
    const TWOMTWO: &str = "2 * 2";
    const TWODTWO: &str = "2 / 2";
    const TWOETWO: &str = "2^2";
    const TWOQTWO: &str = "2 = 2";

    const TWOPTWO_R: &str = "4";
    const TWOSTWO_R: &str = "0";
    const TWOMTWO_R: &str = "4";
    const TWODTWO_R: &str = "1";
    const TWOETWO_R: &str = "4";
    const TWOQTWO_R: &str = "true";

    // Test basic single number expression operation
    #[test]
    fn test_op_engine_sneo() {
        let tokenized_expression = parser::parse_str(TWO).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWO);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test addition
    #[test]
    fn test_op_engine_add() {
        let tokenized_expression = parser::parse_str(TWOPTWO).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOPTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test subtraction
    #[test]
    fn test_op_engine_sub() {
        let tokenized_expression = parser::parse_str(TWOSTWO).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOSTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test multiplication
    #[test]
    fn test_op_engine_mul() {
        let tokenized_expression = parser::parse_str(TWOMTWO).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOMTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test division
    #[test]
    fn test_op_engine_div() {
        let tokenized_expression = parser::parse_str(TWODTWO).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWODTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test exponentation
    #[test]
    fn test_op_engine_exp() {
        let tokenized_expression = parser::parse_str(TWOETWO).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOETWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test equality
    #[test]
    fn test_op_engine_eql() {
        let tokenized_expression = parser::parse_str(TWOQTWO).unwrap();

        if let Token::Boolean(num) = simplify(&tokenized_expression) {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOQTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }
}
