//! Built-in functions and function-table type.
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

use std::error::Error;
use crate::number::Number;
use crate::session::Session;
use simple_error::*;

pub enum Function {
    OneArgN(Box<dyn Fn(Session, Number) -> Number>)
}

impl Function {
    pub fn new_one_n(function: impl Fn(Session, Number) -> Number + 'static) -> Self {
        Self::OneArgN(Box::new(function))
    }
}

pub struct FunctionEntry {
    pub id: String,
    pub function: Function
}

impl FunctionEntry {
    pub fn new_one_n(id: String, function: impl Fn(Session, Number) -> Number + 'static) -> Self {
        Self {
            id: id,
            function: Function::new_one_n(function)
        }
    }
}

pub struct FunctionTable {
    functionTable: Vec::<FunctionEntry>,
}

impl FunctionTable {
    pub fn new() -> Self {
        Self {
            functionTable: Vec::<FunctionEntry>::new()
        }
    }

    /// Add function to the function table, fail if it already exists
    pub fn add_one_n(&mut self, id: String, function: impl Fn(Session, Number) -> Number + 'static) -> Result<(), Box<dyn Error>> {
        match self.functionTable.binary_search_by(|i| i.id.cmp(&id)) {
            Ok(_) => {
                bail!("Function \"{}\" already exists!");
            }
            Err(i) => {
                self.functionTable.insert(i, FunctionEntry::new_one_n(id, function))
            }
        }
        Ok(())
    }
}