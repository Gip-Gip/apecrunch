//! Built-in number type.
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

use fraction::BigFraction;
use fraction::BigUint;
use fraction::One;
use fraction::Sign;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::str::FromStr;

/// Type used to represent and operate on all numerical values, currently just a Big Fraction.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Number {
    /// Fractional representation of the number.
    fraction: BigFraction,
}

impl Number {
    /// Converts a string to a number.
    ///
    pub fn from_str(string: &str) -> Result<Self, Box<dyn Error>> {
        let fraction = BigFraction::from_str(string)?;

        Ok(Number { fraction: fraction })
    }

    /// Returns -1 as a number.
    ///
    pub fn neg_one() -> Self {
        Self {
            fraction: BigFraction::new_raw_signed(Sign::Minus, BigUint::one(), BigUint::one()),
        }
    }

    /// Renders the number to a string.
    ///
    /// Postfixes three dots to indicate there's a loss of precision when rendering.
    ///
    pub fn to_string(&self, prec: usize) -> String {
        // Tell the user that there is more precision than displayed
        // I'll eventually think of a better way to see if we should print three dots...
        let base_str = format!("{num:.prec$}", num = self.fraction, prec = prec);
        let ext_str = format!("{num:.prec$}", num = self.fraction, prec = prec + 1);

        if ext_str.len() > base_str.len() {
            return format!("{}...", base_str);
        }

        base_str
    }

    /// Makes this number negative
    ///
    pub fn negative(&self) -> Number {
        Number {
            fraction: &self.fraction * &Self::neg_one().fraction,
        }
    }

    /// Adds this number to another number.
    ///
    pub fn add(&self, other: &Number) -> Number {
        Number {
            fraction: &self.fraction + &other.fraction,
        }
    }

    /// Subtracts a number from this number.
    ///
    pub fn subtract(&self, other: &Number) -> Number {
        Number {
            fraction: &self.fraction - &other.fraction,
        }
    }

    /// Multiplies this number by another number.
    ///
    pub fn multiply(&self, other: &Number) -> Number {
        Number {
            fraction: &self.fraction * &other.fraction,
        }
    }

    /// Divides this number by another number.
    ///
    pub fn divide(&self, other: &Number) -> Number {
        Number {
            fraction: &self.fraction / &other.fraction,
        }
    }
    /// Raises this number to the power of another number.
    ///
    /// Very bad placeholder exponent function, will get with devs of the fraction crate to add .pow and .sqrt functions to the BigFraction struct.
    ///
    pub fn exponent(&self, other: &Number) -> Number {
        let mut num = self.fraction.clone();

        let mut exp = other.fraction.numer().unwrap().clone();

        // Power the number by the numerator of the exponent.
        while !exp.is_one() {
            num = &num * &self.fraction;
            exp -= &BigUint::one();
        }

        // Root the number by the denominator of the exponent.

        let powered = Number { fraction: num };

        let root = Number {
            fraction: BigFraction::new_raw(
                other.fraction.denom().unwrap().clone(),
                BigUint::from(1u32),
            ),
        };

        Self::root(&powered, &root)
    }

    /// Gets the nth root of this number.
    ///
    /// Hilariously awful root function, temporary placeholder. **VERY SLOW AND VERY INACCURATE.**
    ///
    pub fn root(&self, other: &Number) -> Number {
        if other.fraction > BigFraction::one() {
            let mut i = 100;
            let mut result = BigFraction::new(1u32, 1u32);

            while i > 0 {
                while (&result * &result) > self.fraction {
                    let numer = result.numer().unwrap().clone();
                    let denom = result.denom().unwrap().clone();

                    result = BigFraction::new_raw(numer, denom + 1u32);
                }

                while (&result * &result) < self.fraction {
                    let numer = result.numer().unwrap().clone();
                    let denom = result.denom().unwrap().clone();

                    result = BigFraction::new_raw(numer + 1u32, denom);
                }

                if (&result * &result) == self.fraction {
                    return Number { fraction: result };
                }

                i -= 1;
            }

            return Number { fraction: result };
        }

        Number {
            fraction: self.fraction.clone(),
        }
    }
}
