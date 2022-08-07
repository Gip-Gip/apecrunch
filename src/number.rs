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

        Self::root(&powered, &root, prec)
    }

    /// Gets the nth root of this number.
    ///
    /// Hilariously awful root function, temporary placeholder. **VERY SLOW AND VERY INACCURATE.**
    ///
    pub fn root(&self, other: &Number, prec: u32) -> Number {
        let mut i = other.fraction.clone();
        let mut result = self.clone();
        while i > BigFraction::one() {
            result = result.sqrt(prec);
            i -= BigFraction::one();
        }

        result
    }

    /// Gets the square root of a number
    ///
    pub fn sqrt(&self, prec: u32) -> Self {
        lazy_static! {
            static ref ONE: BigFraction = BigFraction::one();
            static ref TEN_BIGINT: BigUint = 10u8.into();
            static ref ONE_O_TWO: BigFraction = BigFraction::new(1u8, 2u8);
            static ref ONE_O_EIGHT: BigFraction = BigFraction::new(1u8, 8u8);
            static ref ONE_O_SIXTEEN: BigFraction = BigFraction::new(1u8, 16u8);
            static ref FIVE_O_128: BigFraction = BigFraction::new(5u8, 128u8);
            static ref SEVEN_O_256: BigFraction = BigFraction::new(7u8, 256u16);
        }

        let min_move = BigFraction::new(1u8, TEN_BIGINT.pow(prec + 1));

        let a = &self.fraction;

        let mut x = &*ONE_O_TWO * &a;

        let mut last_move = BigFraction::one();

        // Reference hell
        //
        // Simplifies down to
        //
        // h = 1-a / x^2
        // x = x * (1 - h * (1/2 + h * (1/8 + h * (1/16 * h * (5/128 + h * 7/256)))))
        //
        // Will compute until every digit up to the precision is 100% accurate, in theory
        while &last_move > &min_move {
            last_move = x.clone();
            let h = &*ONE - &(a / &(&x * &x));
            x = &x
                * &(&*ONE
                    - &(&h
                        * &(&*ONE_O_TWO
                            + &(&h
                                * &(&*ONE_O_EIGHT
                                    + &(&h
                                        * &(&*ONE_O_SIXTEEN
                                            * &h
                                            * (&*FIVE_O_128 + &(&h * &*SEVEN_O_256)))))))));
            last_move = (&x - &last_move).abs();
            x = Self::round_denom(x, TEN_BIGINT.pow(prec + 1));
        }

        return Self { fraction: x };
    }

    /// Rounds the denominator for quicker calculations which don't need perfect accuracy
    ///
    fn round_denom(fract: BigFraction, denom: BigUint) -> BigFraction {
        let divisor = fract.denom().unwrap() / &denom;

        if divisor > BigUint::zero() {
            BigFraction::new(fract.numer().unwrap() / divisor, denom)
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

        let fract3 = Number::round_denom(fract1, 4u8.into());

        assert_eq!(fract2, fract3);
    }

    #[test]
    fn test_number_simplify() {
        let fract1 = BigFraction::new(1u8, 4u8);
        let fract2 = BigFraction::new(3u8, 16u8) + BigFraction::new(1u8, 16u8);

        assert_eq!(fract1, fract2);
    }
}
