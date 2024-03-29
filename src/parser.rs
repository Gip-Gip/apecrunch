//! Parsing functions and other useful structs.
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

use crate::number::Number;
use crate::session::Session;
use crate::variable::Variable;
use lazy_static::*;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use simple_error::*;
use std::error::Error;
use std::str::FromStr;
use uuid::Uuid;

/// Represents either a single parser token or an entire tokenized expression.
///
/// 2+2 would parse down roughly to Add(Number(2), Number(2)).
///
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Token {
    /// Exponent token, parsed from "^".
    Exponent(Box<Token>, Box<Token>),
    /// Multiply token, parse from "*".
    Multiply(Box<Token>, Box<Token>),
    /// Divide token, parsed from "/".
    Divide(Box<Token>, Box<Token>),
    /// Add token, parsed from "+".
    Add(Box<Token>, Box<Token>),
    /// Subtract token, parsed from "-".
    Subtract(Box<Token>, Box<Token>),
    /// Equality token, parsed from "=".
    Equality(Box<Token>, Box<Token>),
    /// Parenthesis token, parsed from "()".
    Parenthesis(Box<Token>),
    /// Answer token, parsed from an inverse offset preceded by an @
    Answer(Uuid),
    /// Number token, parsed from any number 0-9.
    Number(Number),
    /// Variable token, holds the symbol of a variable along with the value at parsing
    Variable(Box<Variable>),
    /// Negative token, parsed from any negative sign
    Negative(Box<Token>),
    /// Boolean token, not currently parsed but can be returned from simplify functions.
    Boolean(bool),
    /// Store token, used to store values into variables. First string is the id of the variable, second are the tokens to be stored in the variable
    Store(String, Box<Token>),
}

impl Token {
    /// Converts entire tokenized expressions into strings recursively.
    ///
    pub fn to_string(&self, session: &Session) -> String {
        match self {
            Token::Exponent(left, right) => {
                format!("{}^{}", left.to_string(session), right.to_string(session))
            }
            Token::Multiply(left, right) => {
                format!("{} * {}", left.to_string(session), right.to_string(session))
            }
            Token::Divide(left, right) => {
                format!("{} / {}", left.to_string(session), right.to_string(session))
            }
            Token::Add(left, right) => {
                format!("{} + {}", left.to_string(session), right.to_string(session))
            }
            Token::Subtract(left, right) => {
                format!("{} - {}", left.to_string(session), right.to_string(session))
            }
            Token::Equality(left, right) => {
                format!("{} = {}", left.to_string(session), right.to_string(session))
            }
            Token::Parenthesis(expression) => {
                format!("( {} )", expression.to_string(session))
            }
            Token::Answer(uuid) => {
                if let Some(inv_index) = session.get_inv_index_from_uuid(uuid) {
                    return format!("@{}", inv_index);
                }

                "@!".to_string()
            }
            Token::Number(number) => number.to_string(session.decimal_places),
            Token::Negative(expression) => {
                format!("-{}", expression.to_string(session))
            }
            Token::Variable(variable) => {
                format!("{}", variable.id)
            }
            Token::Boolean(boolean) => boolean.to_string(),
            Token::Store(id, expression) => {
                format!("{} -> {}", expression.to_string(session), id)
            }
        }
    }
}

/// Reverse order of operations.
///
/// **NOT PUBLIC.**
///
const ORDER_OF_OPS: [&str; 9] = ["->", "=", "-", "+", "/", "*", NEG_SYMBOL, "^", "@"];

/// Internal symbol for -1, for parser uses only
///
/// **NOT PUBLIC.**
///
const NEG_SYMBOL: &str = "\x26";

/// Strips a string of whitespace, makes sure it's not empty, and runs the string through parse()!
///
/// Throws a simple error if the expression is empty.
///
pub fn parse_str(string: &str, session: &mut Session) -> Result<Token, Box<dyn Error>> {
    // Regex definitions n stuff
    lazy_static! {
        static ref NEGATIVE_RE: Regex = Regex::new(r"(?P<a>^|[(=\-\+/\*\^])(?P<b>-)").unwrap(); // Used to see if there are negative numbers in the string
        static ref COMMENT_RE: Regex = Regex::new(r"#.*").unwrap();
    }

    let mut cleaned_string: String = string.chars().filter(|c| !c.is_whitespace()).collect();

    cleaned_string = COMMENT_RE.replace_all(&cleaned_string, "").to_string();

    if cleaned_string.len() == 0 {
        bail!("Empty Expression!");
    }

    // Replace negative sign with ascii 0x26, for parsing reasons
    cleaned_string = NEGATIVE_RE
        .replace_all(&cleaned_string, format!("$a{}", NEG_SYMBOL))
        .to_string();

    Ok(parse(&cleaned_string, session)?)
}

/// Parses a string recursively, breaking it down into Tokens.
///
/// Returns a simple error if the expression is invalid or incomplete.
///
/// **NOTE: THIS FUNCTION ONLY ACCEPTS WHITESPACE FREE STRINGS.**
///
/// **NOT PUBLIC. USE parse_str() INSTEAD.**
///
fn parse(string: &str, session: &mut Session) -> Result<Token, Box<dyn Error>> {
    // Regex definitions n stuff
    lazy_static! {
        static ref VAR_NAMECHECK_RE: Regex = Regex::new(r"[@=\-\+/\*\^>\u0026]").unwrap(); // Used to see if a variable name is valid; it shouldn't contain any operators
    }

    for opcode in ORDER_OF_OPS {
        // If so...
        let op_index = match_outside_parenthesis(string, opcode)?;

        if op_index.is_some() {
            // Split the string at the operator...
            let splitpoint = op_index.unwrap();

            // If there is nothing to the left or right of the operator, produce an error(unless the operator is the negative sign or an answer sign)...
            if splitpoint + 1 == string.len()
                || (splitpoint == 0 && (opcode != NEG_SYMBOL && opcode != "@"))
            {
                bail!("Incomplete Expression: {}", string);
            }

            // Furthermore, if there is anything left of the negative sign more parsing needs to be done...
            if splitpoint != 0 && opcode == NEG_SYMBOL {
                continue;
            }

            // Separate the string into a left and right side...
            let left = string[..splitpoint].to_string();
            let right = string[splitpoint + opcode.len()..].to_string();

            // Parse the left and right sides recursively...
            return match opcode {
                "@" => {
                    if let Some(entry) = session.get_entry_inv_index(usize::from_str(&right)?) {
                        return Ok(Token::Answer(entry.entry_uuid));
                    }

                    bail!("Invalid answer {}!", right);
                }
                "^" => Ok(Token::Exponent(
                    Box::new(parse(&left, session)?),
                    Box::new(parse(&right, session)?),
                )),
                "-" => Ok(Token::Subtract(
                    Box::new(parse(&left, session)?),
                    Box::new(parse(&right, session)?),
                )),
                "+" => Ok(Token::Add(
                    Box::new(parse(&left, session)?),
                    Box::new(parse(&right, session)?),
                )),
                "/" => Ok(Token::Divide(
                    Box::new(parse(&left, session)?),
                    Box::new(parse(&right, session)?),
                )),
                "*" => Ok(Token::Multiply(
                    Box::new(parse(&left, session)?),
                    Box::new(parse(&right, session)?),
                )),
                NEG_SYMBOL => Ok(Token::Negative(Box::new(parse(&right, session)?))),
                "=" => Ok(Token::Equality(
                    Box::new(parse(&left, session)?),
                    Box::new(parse(&right, session)?),
                )),
                "->" => {
                    // Make sure the variable doesn't contain any operators
                    if VAR_NAMECHECK_RE.is_match(&right) {
                        bail!("Invalid variable name {}!", right);
                    }

                    Ok(Token::Store(right, Box::new(parse(&left, session)?)))
                }
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
    if string.chars().next().unwrap().is_numeric() {
        return Ok(Token::Number(Number::from_str(string)?));
    }

    // If the string is a variable...
    if string.chars().next().unwrap().is_alphabetic() {
        return Ok(Token::Variable(Box::new(session.vartable.get(string)?)));
    }

    // If the string is an expression surrounded in parenthesis...
    if string.chars().next().unwrap() == '(' {
        return Ok(Token::Parenthesis(Box::new(parse(
            &string[1..string.len() - 1],
            session,
        )?)));
    }

    // At the moment nothing else is supported so we bail!
    bail!("Invalid Expression: {}", string);
}

/// Searches for the given substring outside of parenthesis.
///
/// Returns the index of the substring if found, None if otherwise.
///
/// Also returns a simple error if there is an unmatched pair of parenthesis.
///
pub fn match_outside_parenthesis(
    string: &str,
    substring: &str,
) -> Result<Option<usize>, Box<dyn Error>> {
    let mut nest_level = 0;
    let mut index: usize = 0;
    let mut compare_string = String::with_capacity(substring.len());

    for (i, character) in string.chars().enumerate() {
        if character == '(' {
            // Clear the compare string and set the index back to zero if we encounter a parenthesis mid match.
            if index != 0 {
                compare_string.clear();
                index = 0;
            }

            // Increment the nest level if we hit an opening parenthesis.
            nest_level += 1;
        }

        // Decrement the nest level if we hit a closing parenthesis.
        if character == ')' {
            nest_level -= 1;
        }

        // If we're currently not nested in any parenthesis...
        if nest_level == 0 {
            // Push the current character to the compare string...
            compare_string.push(character);

            // And if the two strings are starting to look equal...
            if compare_string == substring[..compare_string.len()] {
                // And the index hasn't been set yet, set the index to the current i.
                if index == 0 {
                    index = i;
                }

                // Or if the strings are equal, return the index
                if compare_string.len() == substring.len() {
                    return Ok(Some(index));
                }
            }
            // Otherwise, sadly we haven't found a match so we must return the index to zero and clear the compare string.
            else {
                index = 0;
                compare_string.clear();
            }
        }
    }

    // If the nest level isn't zero something is wrong with the expression...
    if nest_level > 0 {
        bail!("Forgot to close parenthesis!");
    }

    if nest_level < 0 {
        bail!("Too many closing parenthesis!");
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = "2";
    const X: &str = "x";
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
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_res = parse_str(TWO, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize variables
    #[test]
    fn test_parser_var() {
        let tokenized_expression_ref = Token::Variable(Box::new(Variable::new(
            "x",
            Token::Number(Number::neg_one()),
        )));
        let varvalue = Token::Number(Number::neg_one());

        let mut session = Session::_new_test().unwrap();

        let variable = Variable::new("x", varvalue);

        session.vartable.add(variable).unwrap();

        let tokenized_expression_res = parse_str(X, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can reject bad variable names
    #[test]
    fn test_parser_var_name_check() {
        let mut session = Session::_new_test().unwrap();

        for opcode in ORDER_OF_OPS {
            let bad_variable: String = format!("2->foo{}bar", opcode);

            parse_str(&bad_variable, &mut session).unwrap_err();
        }
    }

    // Test to make sure the parser can recognize add operations
    #[test]
    fn test_parser_add() {
        let expression = TWOPTWO;
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_ref = Token::Add(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize add operations
    #[test]
    fn test_parser_sub() {
        let expression = TWOSTWO;
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_ref = Token::Subtract(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize subtraction operations
    #[test]
    fn test_parser_mul() {
        let expression = TWOMTWO;
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_ref = Token::Multiply(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize division operations
    #[test]
    fn test_parser_div() {
        let expression = TWODTWO;
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_ref = Token::Divide(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize exponent operations
    #[test]
    fn test_parser_exp() {
        let expression = TWOETWO;
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_ref = Token::Exponent(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }

    // Test to make sure the parser can recognize equality operations
    #[test]
    fn test_parser_eql() {
        let expression = TWOQTWO;
        let mut session = Session::_new_test().unwrap();

        let tokenized_expression_ref = Token::Equality(
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
            Box::new(Token::Number(Number::from_str(TWO).unwrap())),
        );

        let tokenized_expression_res = parse_str(expression, &mut session).unwrap();

        // Make sure the reference is equal to the result
        assert_eq!(tokenized_expression_ref, tokenized_expression_res);
    }
}
