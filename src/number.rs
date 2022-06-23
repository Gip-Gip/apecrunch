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
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// Placeholder number struct, will be converted to a fractional number later...
pub struct Number {
    fraction: BigFraction,
}

impl Number {
    pub fn from_str(string: &str) -> Result<Self, Box<dyn Error>> {
        let fraction = BigFraction::from_str(string)?;

        return Ok(Number { fraction: fraction });
    }

    pub fn to_string(&self) -> String {
        return format!("{:.6}", self.fraction);
    }

    pub fn add(&self, other: &Number) -> Number {
        return Number {
            fraction: &self.fraction + &other.fraction,
        };
    }

    pub fn subtract(&self, other: &Number) -> Number {
        return Number {
            fraction: &self.fraction - &other.fraction,
        };
    }

    pub fn multiply(&self, other: &Number) -> Number {
        return Number {
            fraction: &self.fraction * &other.fraction,
        };
    }

    pub fn divide(&self, other: &Number) -> Number {
        return Number {
            fraction: &self.fraction / &other.fraction,
        };
    }

    // Very bad placeholder exponent function, will get with devs of the fraction crate to add .pow and .sqrt functions to the BigFraction struct
    pub fn exponent(&self, other: &Number) -> Number {
        let mut num = self.fraction.clone();

        let mut exp = other.fraction.numer().unwrap().clone();

        // Power the number by the numerator of the exponent
        while !exp.is_one() {
            num = &num * &self.fraction;
            exp -= &BigUint::one();
        }

        // Root the number by the denominator of the exponent

        let powered = Number { fraction: num };

        let root = Number {
            fraction: BigFraction::new_raw(
                other.fraction.denom().unwrap().clone(),
                BigUint::from(1u32),
            ),
        };

        return Self::root(&powered, &root);
    }

    // Hilariously awful root function, temporary placeholder
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

        return Number {
            fraction: self.fraction.clone(),
        };
    }
}
