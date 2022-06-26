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

use crate::number::Number;
use lazy_static::*;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use simple_error::*;
use std::error::Error;

// parser::Token - enum for tokens
//
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Token {
    Exponent(Box<Token>, Box<Token>), // "^"
    Multiply(Box<Token>, Box<Token>), // "*"
    Divide(Box<Token>, Box<Token>),   // "/"
    Add(Box<Token>, Box<Token>),      // "+"
    Subtract(Box<Token>, Box<Token>), // "-"
    Equality(Box<Token>, Box<Token>), // "="
    Parenthesis(Box<Token>),          // ()
    ParenthesisNeg(Box<Token>),       // -()
    Number(Number),                   // 0-9
    Boolean(bool),                    // true or false
}

impl Token {
    pub fn to_string(&self, prec: usize) -> String {
        match self {
            Token::Exponent(left, right) => {
                return format!("{}^{}", left.to_string(prec), right.to_string(prec));
            }
            Token::Multiply(left, right) => {
                return format!("{} * {}", left.to_string(prec), right.to_string(prec));
            }
            Token::Divide(left, right) => {
                return format!("{} / {}", left.to_string(prec), right.to_string(prec));
            }
            Token::Add(left, right) => {
                return format!("{} + {}", left.to_string(prec), right.to_string(prec));
            }
            Token::Subtract(left, right) => {
                return format!("{} - {}", left.to_string(prec), right.to_string(prec));
            }
            Token::Equality(left, right) => {
                return format!("{} = {}", left.to_string(prec), right.to_string(prec));
            }
            Token::Parenthesis(expression) => {
                return format!("( {} )", expression.to_string(prec));
            }
            Token::ParenthesisNeg(expression) => {
                return format!("-( {} )", expression.to_string(prec));
            }
            Token::Number(number) => {
                return number.to_string(prec);
            }
            Token::Boolean(boolean) => {
                return boolean.to_string();
            }
        }
    }
}

const ORDER_OF_OPS: [&str; 6] = ["=", "-", "+", "/", "*", "^"]; // Order of operations, reversed

// parser::parse_str - parse a string into tokens
//
// ARGUMENTS:
//  string: &str - string to parse
//
// DESCRIPTION:
//  This function strips the string of whitespace and parses it into tokens through the recursive function parse()
pub fn parse_str(string: &str) -> Result<Token, Box<dyn Error>> {
    let cleaned_string: String = string.chars().filter(|c| !c.is_whitespace()).collect();

    if cleaned_string.len() == 0 {
        bail!("Empty Expression!");
    }

    return Ok(parse(&cleaned_string)?);
}

// parser::parse - parse a cleaned string into tokens
//
// ARGUMENTS:
//  string: &str - string to parse
//
// DESCRIPTION:
//  This function parses a string into tokens recusively.
//  **THIS FUNCTION ONLY ACCEPTS WHITESPACE FREE STRINGS**
fn parse(string: &str) -> Result<Token, Box<dyn Error>> {
    // Regex definitions for various parsing uses
    lazy_static! {
        static ref NEGATIVE_RE: Regex = Regex::new(r"(?P<a>^|[=\-\+/\*\^])(?P<b>-)").unwrap(); // Used to see if there are negative numbers in the string
    }

    let neg_indicies = NEGATIVE_RE.find_iter(string);
    let working_string = &NEGATIVE_RE.replace_all(string.clone(), "$a");

    for opcode in ORDER_OF_OPS {
        // If so...
        let op_index = match_outside_parenthesis(working_string, opcode)?;

        if op_index.is_some() {
            // Split the string at the operator...
            let mut splitpoint = op_index.unwrap();

            // Make up for any removed characters in the string
            for i in neg_indicies {
                if splitpoint > i.start() {
                    splitpoint += 1;
                }
            }

            // If there is nothing to the left or right of the operator, produce an error...
            if splitpoint + 1 == string.len() || splitpoint == 0 {
                bail!("Incomplete Expression: {}", string);
            }

            // Separate the string into a left and right side...
            let left = string[..splitpoint].to_string();
            let right = string[splitpoint + 1..].to_string();

            // Parse the left and right sides recursively...
            return match opcode {
                "^" => Ok(Token::Exponent(
                    Box::new(parse(&left)?),
                    Box::new(parse(&right)?),
                )),
                "-" => Ok(Token::Subtract(
                    Box::new(parse(&left)?),
                    Box::new(parse(&right)?),
                )),
                "+" => Ok(Token::Add(
                    Box::new(parse(&left)?),
                    Box::new(parse(&right)?),
                )),
                "/" => Ok(Token::Divide(
                    Box::new(parse(&left)?),
                    Box::new(parse(&right)?),
                )),
                "*" => Ok(Token::Multiply(
                    Box::new(parse(&left)?),
                    Box::new(parse(&right)?),
                )),
                "=" => Ok(Token::Equality(
                    Box::new(parse(&left)?),
                    Box::new(parse(&right)?),
                )),
                // It is entrely possible I am a terrible programmer and I forgot to implement all the operators in the ORDER_OF_OPS table...
                _ => {
                    panic!("\n\nFatal Oopsiedaisies!\n\n\tOperator found in table but no code to handle it: {}\n\n", opcode);
                }
            };
        }
    }

    // If the string doesn't contain an operator we need to figure out what it is...
    //

    // If the string is a number...
    if working_string.chars().next().unwrap().is_numeric() {
        return Ok(Token::Number(Number::from_str(string)?));
    }

    // If the string is an expression surrounded in parenthesis...
    if working_string.chars().next().unwrap() == '(' {
        if NEGATIVE_RE.is_match(string) {
            return Ok(Token::ParenthesisNeg(Box::new(parse(
                &string[2..string.len() - 1],
            )?)));
        }

        return Ok(Token::Parenthesis(Box::new(parse(
            &string[1..string.len() - 1],
        )?)));
    }

    // At the moment nothing else is supported so we bail!
    bail!("Invalid Expression: {}", string);
}

pub fn match_outside_parenthesis(
    string: &str,
    substring: &str,
) -> Result<Option<usize>, Box<dyn Error>> {
    let mut nest_level = 0;
    let mut index: usize = 0;
    let mut compare_string = String::with_capacity(substring.len());

    for (i, character) in string.chars().enumerate() {
        if character == '(' {
            if index != 0 {
                compare_string.clear();
                index = 0;
            }

            nest_level += 1;
        }

        if character == ')' {
            nest_level -= 1;
        }

        if nest_level == 0 {
            compare_string.push(character);

            if compare_string == substring[..compare_string.len()] {
                if index == 0 {
                    index = i;
                }

                if compare_string.len() == substring.len() {
                    return Ok(Some(index));
                }
            } else {
                index = 0;
                compare_string.clear();
            }
        }
    }

    if nest_level > 0 {
        bail!("Forgot to close parenthesis!");
    }

    if nest_level < 0 {
        bail!("Too many closing parenthesis!");
    }

    return Ok(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = "2";
    const TWOPTWO: &str = "2 + 2";
    const TWOSTWO: &str = "2 - 2";
    const TWOMTWO: &str = "2 * 2";
    const TWODTWO: &str = "2 / 2";
    const TWOETWO: &str = "2^2";
    const TWOQTWO: &str = "2 = 2";

    // Test to make sure the parser can recognize numbers
    #[test]
    fn test_parser_num() {
        let tokenized_expression_ref = Token::Number(Number::from_str(TWO).unwrap());

        let tokenized_expression_res = parse_str(TWO).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize add operations
    #[test]
    fn test_parser_add() {
        let expression = TWOPTWO;

        let tokenized_expression_ref = Token::Add(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize add operations
    #[test]
    fn test_parser_sub() {
        let expression = TWOSTWO;

        let tokenized_expression_ref = Token::Subtract(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize subtraction operations
    #[test]
    fn test_parser_mul() {
        let expression = TWOMTWO;

        let tokenized_expression_ref = Token::Multiply(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize division operations
    #[test]
    fn test_parser_div() {
        let expression = TWODTWO;

        let tokenized_expression_ref = Token::Divide(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize exponent operations
    #[test]
    fn test_parser_exp() {
        let expression = TWOETWO;

        let tokenized_expression_ref = Token::Exponent(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize equality operations
    #[test]
    fn test_parser_eql() {
        let expression = TWOQTWO;

        let tokenized_expression_ref = Token::Equality(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }
}
