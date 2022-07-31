//! Operation-engine. Functions for performing operations on parsed expressions.
//!

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
use crate::variable::VarTable;
use crate::variable::Variable;
use std::error::Error;

/// Simplifies the given parser tokens and asserts they are equal to the unsimplified parser tokens, returning both in an equality token.alloc
///
/// For example, 2+2 would be equal to 4.
///
pub fn get_equality(tokens: &Token, vartable: &mut VarTable) -> Result<Token, Box<dyn Error>> {
    Ok(Token::Equality(
        Box::new(tokens.clone()),
        Box::new(simplify(tokens, vartable)?),
    ))
}

/// Recursively simplifies an expression, performing various operations like multiplication, division, etc. etc.
///
/// For example, 2+2 would simplify into 4.
///
pub fn simplify(token: &Token, vartable: &mut VarTable) -> Result<Token, Box<dyn Error>> {
    match token {
        // Almost all of these match cases are the same, understand this one and you understand them all...
        Token::Multiply(left, right) => {
            let left_result = simplify(left, vartable)?; // Recursively simplify the left side.
            let right_result = simplify(right, vartable)?; // Recursively simplify the right side.

            // If both sides are numbers, operate on them and return a number token.
            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.multiply(&right_number)));
                    // In this case we multiply the two.
                }
            }

            // Otherwise it cannot be further simplified, and we must return a multiply token.
            Ok(Token::Multiply(
                Box::new(left_result),
                Box::new(right_result),
            ))
        }

        Token::Divide(left, right) => {
            let left_result = simplify(left, vartable)?;
            let right_result = simplify(right, vartable)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.divide(&right_number)));
                }
            }

            Ok(Token::Divide(Box::new(left_result), Box::new(right_result)))
        }

        Token::Add(left, right) => {
            let left_result = simplify(left, vartable)?;
            let right_result = simplify(right, vartable)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.add(&right_number)));
                }
            }

            Ok(Token::Add(Box::new(left_result), Box::new(right_result)))
        }

        Token::Subtract(left, right) => {
            let left_result = simplify(left, vartable)?;
            let right_result = simplify(right, vartable)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.subtract(&right_number)));
                }
            }

            Ok(Token::Subtract(
                Box::new(left_result),
                Box::new(right_result),
            ))
        }

        Token::Exponent(left, right) => {
            let left_result = simplify(left, vartable)?;
            let right_result = simplify(right, vartable)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.exponent(&right_number)));
                }
            }

            Ok(Token::Exponent(
                Box::new(left_result),
                Box::new(right_result),
            ))
        }

        Token::Equality(left, right) => {
            let left_result = simplify(left, vartable)?;
            let right_result = simplify(right, vartable)?;

            Ok(Token::Boolean(left_result == right_result))
        }

        Token::Parenthesis(expression) => simplify(expression, vartable),

        Token::Number(_number) => Ok(token.clone()),

        Token::Boolean(_truth) => Ok(token.clone()),

        Token::Variable(variable) => Ok(variable.tokens.clone()),

        Token::Negative(expression) => {
            let result = simplify(expression, vartable)?;

            if let Token::Number(number) = &result {
                return Ok(Token::Number(number.negative()));
            }

            Ok(Token::Negative(Box::new(result)))
        }

        Token::Store(id, tokens) => {
            let simplified_tokens = simplify(tokens, vartable)?;
            let variable = Variable::new(id, simplified_tokens.clone());
            vartable.store(variable)?;

            Ok(simplified_tokens)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::variable::VarTable;

    const TWO: &str = "2";
    const TWOPTWO: &str = "2 + 2";
    const TWOSTWO: &str = "2 - 2";
    const TWOMTWO: &str = "2 * 2";
    const TWODTWO: &str = "2 / 2";
    const TWOETWO: &str = "2^2";
    const TWOQTWO: &str = "2 = 2";
    const TWOSTORE: &str = "2 -> x";
    const TWORET: &str = "x";

    const TWOPTWO_R: &str = "4";
    const TWOSTWO_R: &str = "0";
    const TWOMTWO_R: &str = "4";
    const TWODTWO_R: &str = "1";
    const TWOETWO_R: &str = "4";
    const TWOQTWO_R: &str = "true";
    const TWOSTORE_R: &str = "2";
    const TWORET_R: &str = "2";

    // Test basic single number expression operation
    #[test]
    fn test_op_engine_sneo() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWO, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWO);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test addition
    #[test]
    fn test_op_engine_add() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWOPTWO, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOPTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test subtraction
    #[test]
    fn test_op_engine_sub() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWOSTWO, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOSTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test multiplication
    #[test]
    fn test_op_engine_mul() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWOMTWO, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOMTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test division
    #[test]
    fn test_op_engine_div() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWODTWO, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWODTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test exponentation
    #[test]
    fn test_op_engine_exp() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWOETWO, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOETWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test equality
    #[test]
    fn test_op_engine_eql() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWOQTWO, &mut vartable).unwrap();

        if let Token::Boolean(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOQTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test storing
    #[test]
    fn test_op_engine_store_retrieve() {
        let mut vartable = VarTable::new();
        let tokenized_expression = parser::parse_str(TWOSTORE, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOSTORE_R);
        } else {
            panic!("Didn't return number token!");
        }

        let tokenized_expression = parser::parse_str(TWORET, &mut vartable).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut vartable).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWORET_R);
        } else {
            panic!("Didn't return number token!");
        }
    }
}
