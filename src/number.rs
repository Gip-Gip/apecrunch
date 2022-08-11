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
use fraction::Signed;
use fraction::Zero;
use lazy_static::*;
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
    pub fn to_string(&self, prec: u32) -> String {
        // Tell the user that there is more precision than displayed
        // I'll eventually think of a better way to see if we should print three dots...
        let base_str = format!("{num:.prec$}", num = self.fraction, prec = prec as usize);
        let ext_str = format!(
            "{num:.prec$}",
            num = self.fraction,
            prec = (prec as usize + 1)
        );

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
    pub fn exponent(&self, other: &Number, prec: u32) -> Number {
        match &self.fraction {
            BigFraction::Infinity(_) | BigFraction::NaN => return self.clone(),
            _ => {}
        }

        let result = Self {
            fraction: Self::pow(&self.fraction, &other.fraction),
        };

        let root = other.fraction.denom().unwrap().clone();

        if root == BigUint::one() {
            result
        } else {
            let root_num = Self {
                fraction: BigFraction::new_raw(root, 1u8.into()),
            };

            Self::root(&result, &root_num, prec)
        }
    }

    fn pow(num: &BigFraction, pow: &BigFraction) -> BigFraction {
        match num {
            BigFraction::Infinity(_) | BigFraction::NaN => return num.clone(),
            _ => {}
        }

        let mut i = pow.numer().unwrap().clone();
        let mut result = num.clone();

        while i > BigUint::one() {
            result = num * &result;
            i -= BigUint::one();
        }

        if pow.is_negative() {
            BigFraction::one() / result
        } else {
            result
        }
    }

    /// Gets the nth root of this number.
    ///
    pub fn root(&self, other: &Number, prec: u32) -> Number {
        lazy_static! {
            static ref ONE: BigFraction = BigFraction::one();
            static ref TWO: BigFraction = 2u8.into();
            static ref TEN_BIGINT: BigUint = 10u8.into();
        }

        match &self.fraction {
            BigFraction::Infinity(_) | BigFraction::NaN => return self.clone(),
            _ => {
                if self.fraction < BigFraction::zero() {
                    // Return NaN if we're trying to get the square root of a negative, to be implemented!
                    return Self {
                        fraction: BigFraction::NaN,
                    };
                }
            }
        }

        let prec_bigint = TEN_BIGINT.pow(prec + 1);

        let min_move = BigFraction::new(1u8, prec_bigint.clone());

        let a = &self.fraction;

        let root = BigFraction::new_raw(other.fraction.numer().unwrap().clone(), 1u8.into());

        let lroot = &root - &BigFraction::one();

        let mut x = a / &*TWO;

        let mut last_move = BigFraction::one();

        // Will compute until every digit up to the precision is 100% accurate, in theory
        //
        // Uses newton method for finding a root
        while &last_move >= &min_move {
            last_move = x.clone();
            x = Self::round_denom(x, &prec_bigint);
            x = &x - &(&(&Self::pow(&x, &root) - a) / &(&root * &Self::pow(&x, &lroot)));
            last_move = (&x - &last_move).abs();
        }

        let result = Self {
            fraction: match other.fraction.is_negative() {
                true => BigFraction::one() / x,
                false => x,
            },
        };

        if other.fraction.denom().unwrap() == &BigUint::one() {
            result
        } else {
            let exp = Number {
                fraction: BigFraction::new_raw(other.fraction.denom().unwrap().clone(), 1u8.into()),
            };

            Self::exponent(&result, &exp, prec)
        }
    }

    /// Rounds the denominator for quicker calculations which don't need perfect accuracy
    ///
    fn round_denom(fract: BigFraction, denom: &BigUint) -> BigFraction {
        let divisor = fract.denom().unwrap() / denom;

        if divisor > BigUint::zero() {
            BigFraction::new(fract.numer().unwrap() / divisor, denom.clone())
        } else {
            fract
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_round_denom() {
        let fract1 = BigFraction::new(3u8, 8u8);
        let fract2 = BigFraction::new(1u8, 4u8);

        let fract3 = Number::round_denom(fract1, &4u8.into());

        assert_eq!(fract2, fract3);
    }

    #[test]
    fn test_number_simplify() {
        let fract1 = BigFraction::new(1u8, 4u8);
        let fract2 = BigFraction::new(3u8, 16u8) + BigFraction::new(1u8, 16u8);

        assert_eq!(fract1, fract2);
    }
}
