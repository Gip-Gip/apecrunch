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

use std::error::Error;
use crate::number::Number;
use simple_error::*;

// parser::Token - enum for tokens
//
#[derive(Debug, Clone, PartialEq)]
pub enum Token
{
    Multiply(Box<Token>, Box<Token>), // "*"
    Divide(Box<Token>, Box<Token>), // "/"
    Add(Box<Token>, Box<Token>), // "+"
    Subtract(Box<Token>, Box<Token>), // "-"
    Equality(Box<Token>, Box<Token>), // "="
    Number(Number), // 0-9
    Boolean(bool) // true or false
}

impl Token
{
    pub fn to_string(&self) -> String
    {
        match self
        {
            Token::Multiply(left, right) =>
            {
                return format!("{} * {}", left.to_string(), right.to_string());
            }
            Token::Divide(left, right) =>
            {
                return format!("{} / {}", left.to_string(), right.to_string());
            }
            Token::Add(left, right) =>
            {
                return format!("{} + {}", left.to_string(), right.to_string());
            }
            Token::Subtract(left, right) =>
            {
                return format!("{} - {}", left.to_string(), right.to_string());
            }
            Token::Equality(left, right) =>
            {
                return format!("{} = {}", left.to_string(), right.to_string());
            }
            Token::Number(number) =>
            {
                return number.to_string();
            }
            Token::Boolean(boolean) =>
            {
                return boolean.to_string();
            }
        }
    }
}

const ORDER_OF_OPS : [char; 5] = ['=', '-', '+', '/', '*']; // Order of operations, reversed

// parser::parse_str - parse a string into tokens
//
// ARGUMENTS:
//  string: &str - string to parse
//
// DESCRIPTION:
//  This function strips the string of whitespace and parses it into tokens through the recursive function parse()
pub fn parse_str(string: &str) -> Result<Token, Box<dyn Error>>
{
    let cleaned_string: String = string.chars().filter(|c| !c.is_whitespace()).collect();

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
fn parse(string: &str) -> Result<Token, Box<dyn Error>>
{
    // First see if the string contains an operator...
    for opcode in ORDER_OF_OPS
    {
        // If so...
        if string.contains(opcode)
        {
            // Split the string at the operator...
            let splitpoint = string.find(opcode).unwrap();

            // If there is nothing to the left or right of the operator, produce an error...
            if splitpoint + 1 == string.len() || splitpoint == 0
            {
                bail!("Incomplete Expression: {}", string);
            }

            // Separate the string into a left and right side...
            let left = string[..splitpoint].to_string();
            let right = string[splitpoint + 1..].to_string();

            // Parse the left and right sides recursively...
            return match opcode
            {
                '-' =>
                {
                    Ok(Token::Subtract
                    (
                        Box::new(parse(&left)?),
                        Box::new(parse(&right)?)
                    ))      
                }
                '+' =>
                {
                    Ok(Token::Add
                    (
                        Box::new(parse(&left)?),
                        Box::new(parse(&right)?)
                    ))
                }
                '/' =>
                {
                    Ok(Token::Divide
                    (
                        Box::new(parse(&left)?),
                        Box::new(parse(&right)?)
                    ))
                }
                '*' =>
                {
                    Ok(Token::Multiply
                    (
                        Box::new(parse(&left)?),
                        Box::new(parse(&right)?)
                    ))
                }
                '=' =>
                {
                    Ok(Token::Equality
                    (
                        Box::new(parse(&left)?),
                        Box::new(parse(&right)?)
                    ))
                }
                // It is entrely possible I am a terrible programmer and I forgot to implement all the operators in the ORDER_OF_OPS table...
                _ =>
                {
                    panic!("Fatal Oopsiedaisies!\n\n\tOperator found in table but no code to handle it: {}", opcode);
                }
            };
        }
    }

    // If the string doesn't contain an operator we need to figure out what it is...
    //

    // If the string is a number...
    if string.chars().next().unwrap().is_numeric()
    {
        return Ok(Token::Number(Number::from_str(string)?));
    }

    // At the moment nothing else is supported so we bail!
    bail!("Invalid Expression: {}", string);
}