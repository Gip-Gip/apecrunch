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

use crate::session::Session;
use crate::parser::Token;
use crate::variable::Variable;
use std::error::Error;
use simple_error::*;

/// Simplifies the given parser tokens and asserts they are equal to the unsimplified parser tokens, returning both in an equality token.alloc
///
/// For example, 2+2 would be equal to 4.
///
pub fn get_equality(
    tokens: &Token,
    session: &mut Session
) -> Result<Token, Box<dyn Error>> {
    Ok(Token::Equality(
        Box::new(tokens.clone()),
        Box::new(simplify(tokens, session)?),
    ))
}

/// Recursively simplifies an expression, performing various operations like multiplication, division, etc. etc.
///
/// For example, 2+2 would simplify into 4.
///
pub fn simplify(
    token: &Token,
    session: &mut Session,
) -> Result<Token, Box<dyn Error>> {
    match token {
        // Almost all of these match cases are the same, understand this one and you understand them all...
        Token::Multiply(left, right) => {
            let left_result = simplify(left, session)?; // Recursively simplify the left side.
            let right_result = simplify(right, session)?; // Recursively simplify the right side.

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
            let left_result = simplify(left, session)?;
            let right_result = simplify(right, session)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.divide(&right_number)));
                }
            }

            Ok(Token::Divide(Box::new(left_result), Box::new(right_result)))
        }

        Token::Add(left, right) => {
            let left_result = simplify(left, session)?;
            let right_result = simplify(right, session)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.add(&right_number)));
                }
            }

            Ok(Token::Add(Box::new(left_result), Box::new(right_result)))
        }

        Token::Subtract(left, right) => {
            let left_result = simplify(left, session)?;
            let right_result = simplify(right, session)?;

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
            let left_result = simplify(left, session)?;
            let right_result = simplify(right, session)?;

            if let Token::Number(left_number) = &left_result {
                if let Token::Number(right_number) = &right_result {
                    return Ok(Token::Number(left_number.exponent(&right_number, session.decimal_places)));
                }
            }

            Ok(Token::Exponent(
                Box::new(left_result),
                Box::new(right_result),
            ))
        }

        Token::Equality(left, right) => {
            let left_result = simplify(left, session)?;
            let right_result = simplify(right, session)?;

            Ok(Token::Boolean(left_result == right_result))
        }

        Token::Parenthesis(expression) => simplify(expression, session),

        Token::Answer(uuid) => {
            if let Some(entry) = session.get_entry_from_uuid(uuid) {
                let entry = entry.clone();
                return simplify(entry.without_equality(), session)
            }
            bail!("Invalid entry uuid {}!", uuid);
        }

        Token::Number(_number) => Ok(token.clone()),

        Token::Boolean(_truth) => Ok(token.clone()),

        Token::Variable(variable) => Ok(variable.tokens.clone()),

        Token::Negative(expression) => {
            let result = simplify(expression, session)?;

            if let Token::Number(number) = &result {
                return Ok(Token::Number(number.negative()));
            }

            Ok(Token::Negative(Box::new(result)))
        }

        Token::Store(id, tokens) => {
            let simplified_tokens = simplify(tokens, session)?;
            let variable = Variable::new(id, simplified_tokens.clone());
            session.vartable.store(variable)?;

            Ok(simplified_tokens)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Session;
use super::*;
    use crate::parser;

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
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWO, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWO);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test addition
    #[test]
    fn test_op_engine_add() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWOPTWO, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOPTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test subtraction
    #[test]
    fn test_op_engine_sub() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWOSTWO, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOSTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test multiplication
    #[test]
    fn test_op_engine_mul() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWOMTWO, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOMTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test division
    #[test]
    fn test_op_engine_div() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWODTWO, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWODTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test exponentation
    #[test]
    fn test_op_engine_exp() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWOETWO, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOETWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test equality
    #[test]
    fn test_op_engine_eql() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWOQTWO, &mut session).unwrap();

        if let Token::Boolean(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(), TWOQTWO_R);
        } else {
            panic!("Didn't return number token!");
        }
    }

    // Test storing
    #[test]
    fn test_op_engine_store_retrieve() {
        let mut session = Session::_new_test().unwrap();
        let tokenized_expression = parser::parse_str(TWOSTORE, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWOSTORE_R);
        } else {
            panic!("Didn't return number token!");
        }

        let tokenized_expression = parser::parse_str(TWORET, &mut session).unwrap();

        if let Token::Number(num) = simplify(&tokenized_expression, &mut session).unwrap() {
            // Assert that the right of the operation is what we expect
            assert_eq!(num.to_string(6), TWORET_R);
        } else {
            panic!("Didn't return number token!");
        }
    }
}
