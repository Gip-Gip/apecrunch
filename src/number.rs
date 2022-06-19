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

use std::str::FromStr;
use fraction::BigFraction;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
// Placeholder number struct, will be converted to a fractional number later...
pub struct Number
{
    fraction: BigFraction,
}

impl Number
{
    pub fn from_str(string: &str) -> Result<Self, Box<dyn Error>>
    {
        let fraction = BigFraction::from_str(string)?;
        
        return Ok(Number{
            fraction: fraction
        });
    }

    pub fn to_string(&self) -> String
    {
        return format!("{:.3}", self.fraction);
    }

    pub fn add(&self, other: &Number) -> Number
    {
        return Number{
            fraction: &self.fraction + &other.fraction
        };
    }

    pub fn subtract(&self, other: &Number) -> Number
    {
        return Number{
            fraction: &self.fraction - &other.fraction
        };
    }

    pub fn multiply(&self, other: &Number) -> Number
    {
        return Number{
            fraction: &self.fraction * &other.fraction
        };
    }

    pub fn divide(&self, other: &Number) -> Number
    {
        return Number{
            fraction: &self.fraction / &other.fraction
        };
    }
}