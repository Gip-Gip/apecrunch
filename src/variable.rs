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
}

/// Struct for the built-in vartable type
///
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VarTable {
    /// Vector of variables
    pub variables: Vec<Variable>,
}

impl VarTable {
    /// Create an empty VarTable
    ///
    pub fn new() -> Self {
        Self {
            variables: Vec::<Variable>::new(),
        }
    }

    /// Merge with another vartable, overwriting all existing variables with ones found in the other vartable
    ///
    pub fn merge(&mut self, vartable: &VarTable) -> Result<(), Box<dyn Error>> {
        for variable in &vartable.variables {
            self.store(variable.to_owned())?;
        }

        Ok(())
    }

    /// Add a variable to the VarTable, fail if the variable exists
    ///
    pub fn add(&mut self, var: Variable) -> Result<(), Box<dyn Error>> {
        match self.variables.binary_search_by(|i| i.id.cmp(&var.id)) {
            Ok(_) => {
                bail!("Variable \"{}\" already found!", var.id);
            }
            Err(i) => {
                self.variables.insert(i, var);
            }
        }

        Ok(())
    }

    /// Remove a variable from the VarTable given just the id, fail if the variable doesn't exist
    ///
    pub fn remove(&mut self, id: &str) -> Result<(), Box<dyn Error>> {
        match self.variables.binary_search_by(|i| i.id.as_str().cmp(&id)) {
            Ok(i) => {
                self.variables.remove(i);
            }
            Err(_) => {
                bail!("Variable \"{}\" not found!", id);
            }
        }

        Ok(())
    }

    /// Store a variable to the VarTable, replacing a variable if it exists with the updated value
    ///
    pub fn store(&mut self, var: Variable) -> Result<(), Box<dyn Error>> {
        match self.variables.binary_search_by(|i| i.id.cmp(&var.id)) {
            Ok(i) => {
                self.variables[i] = var.clone(); // If the variable exists replace it with the new variable
            }
            Err(i) => {
                self.variables.insert(i, var); // Otherwise insert it
            }
        }

        Ok(())
    }

    /// Get a variable from the VarTable given just the id
    ///
    pub fn get(&mut self, id: &str) -> Result<Variable, Box<dyn Error>> {
        match self.variables.binary_search_by(|i| i.id.as_str().cmp(&id)) {
            Ok(i) => Ok(self.variables.get(i).unwrap().clone()),
            Err(_) => {
                bail!("Variable \"{}\" not found!", id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::Number;
    use crate::session::Session;

    #[test]
    fn test_var() {
        let var = Variable::new("x", Token::Number(Number::neg_one()));
        let session = Session::_new_test().unwrap();

        assert_eq!(var.id, "x");
        assert_eq!(var.tokens.to_string(&session), "-1");
    }

    #[test]
    fn test_vartable_add_store_get_remove() {
        let var = Variable::new("x", Token::Number(Number::neg_one()));
        let var2 = Variable::new("x", Token::Number(Number::from_str("2").unwrap()));

        // Create the vartable
        let mut vartable = VarTable::new();

        // Add the variable
        vartable.add(var.clone()).unwrap();

        // Add the variable again, should fail
        vartable.add(var.clone()).unwrap_err();

        // Store the variable, should overwrite the existing variable
        vartable.store(var2.clone()).unwrap();

        // Make sure there are no duplicates

        let mut vartable2 = vartable.clone();

        vartable2.variables.dedup();

        assert_eq!(vartable, vartable2);

        // Retrieve it
        let var3 = vartable.get("x").unwrap();

        // Assert they are equal
        assert_eq!(var3, var2);

        // Remove it
        vartable.remove("x").unwrap();

        // Make sure it's removed(expect error)
        vartable.get("x").unwrap_err();
    }
}
