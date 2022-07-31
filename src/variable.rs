//! Built-in variable and variable-table type
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
use serde::Deserialize;
use serde::Serialize;
use simple_error::*;
use std::error::Error;

/// Struct for the built-in variable type
///
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Variable {
    /// ID
    pub id: String,
    /// Value/tokens assigned to the variable
    pub tokens: Token,
}

impl Variable {
    pub fn new(id: &str, value: Token) -> Self {
        Self {
            id: id.to_owned(),
            tokens: value,
        }
    }

    pub fn to_string(&self) -> String {
        self.id.clone()
    }

    pub fn get_value(&self) -> Token {
        self.tokens.clone()
    }
}

/// Struct for the built-in vartable type
///
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VarTable {
    /// Vector of variables
    pub variables: Vec<Variable>,
}

impl VarTable {
    pub fn new() -> Self {
        Self {
            variables: Vec::<Variable>::new(),
        }
    }
    pub fn add(&mut self, var: Variable) -> Result<(), Box<dyn Error>> {
        if self.variables.iter().find(|&i| i.id == var.id) == None {
            self.variables.push(var);
            Ok(())
        } else {
            bail!("Variable \"{}\" already found!", var.id);
        }
    }

    pub fn remove(&mut self, id: &str) -> Result<(), Box<dyn Error>> {
        let new_variables: Vec<Variable> = self
            .variables
            .clone()
            .into_iter()
            .filter(|i| i.id != id)
            .collect();

        if new_variables.len() == self.variables.len() {
            bail!("Variable \"{}\" not found!", id);
        }

        self.variables = new_variables;

        Ok(())
    }

    pub fn get(&mut self, id: &str) -> Result<Variable, Box<dyn Error>> {
        match self.variables.iter().find(|&i| i.id == id) {
            Some(var) => Ok(var.clone()),
            None => {
                bail!("Variable \"{}\" not found!", id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::Number;

    #[test]
    fn test_var() {
        let var = Variable::new("x", Token::Number(Number::neg_one()));

        assert_eq!(var.to_string(), "x");
        assert_eq!(var.get_value().to_string(0), "-1");
    }

    #[test]
    fn test_vartable_add_get_remove() {
        let var = Variable::new("x", Token::Number(Number::neg_one()));

        // Create the vartable
        let mut vartable = VarTable::new();

        // Add the variable
        vartable.add(var.clone()).unwrap();

        // Add the variable again, should fail
        vartable.add(var.clone()).unwrap_err();

        // Retrieve it
        let var2 = vartable.get("x").unwrap();

        // Assert they are equal
        assert_eq!(var, var2);

        // Remove it
        vartable.remove("x").unwrap();

        // Make sure it's removed(expect error)
        vartable.get("x").unwrap_err();
    }
}
