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

#[derive(Debug, Clone, PartialEq)]
// Placeholder number struct, will be converted to a fractional number later...
pub struct Number
{
    value: i64
}

impl Number
{   
    pub fn from_str(s: &str) -> Result<Self, Box<dyn Error>>
    {
        let value = s.parse::<i64>()?;
        
        return Ok(Number{
            value: value
        });
    }

    pub fn to_string(&self) -> String
    {
        return self.value.to_string();
    }

    pub fn add(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value + other.value
        };
    }

    pub fn subtract(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value - other.value
        };
    }

    pub fn multiply(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value * other.value
        };
    }

    pub fn divide(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value / other.value
        };
    }
}