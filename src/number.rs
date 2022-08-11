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
        let printed_value = BigFraction::from_str(&base_str).unwrap(); // This should always work, right?

        if printed_value != self.fraction {
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
    pub fn exponent(&self, exp: &Number, prec: u32) -> Number {
        // Just clone and return the number if equal to ∞ or NaN
        match &self.fraction {
            BigFraction::Infinity(_) | BigFraction::NaN => return self.clone(),
            _ => {}
        }

        let result = Self {
            fraction: Self::pow(&self.fraction, &exp.fraction),
        };

        let root = exp.fraction.denom().unwrap().clone();

        if root == BigUint::one() {
            result
        } else {
            let root_num = Self {
                fraction: BigFraction::new_raw(root, 1u8.into()),
            };

            Self::root(&result, &root_num, prec)
        }
    }

    /// Raises a BigFraction to the power of another BigFraction, ignoring the denominator of the power
    ///
    /// **PRIVATE FUNCTION**
    ///
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
    pub fn root(&self, root: &Number, prec: u32) -> Number {
        // Define statics so these don't have to be reinitialized every time this function is called
        lazy_static! {
            static ref ONE: BigFraction = BigFraction::one();
            static ref TWO: BigFraction = 2u8.into();
            static ref TEN_BIGINT: BigUint = 10u8.into();
        }

        // Just clone and return the number if equal to ∞ or NaN
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

        let prec_working = TEN_BIGINT.pow(prec + 3); // Precision to round the number to while performing calculations, should be one decimal greater than the min_step

        let min_move = BigFraction::new(1u8, TEN_BIGINT.pow(prec + 2)); // Precision to stop calculations at, should be one decimal greater than the final precision

        let prec_final = TEN_BIGINT.pow(prec + 1); // Final precision is always greater than the printed precision so the number's to_string function knows to print the three dots

        // Algorithm uses Newton's method for calculating roots of numbers
        //
        // Can be represented with the following python code:
        //
        //  def root(a, rt, iter):
        //      lr = rt - 1;
        //      x = a / 2
        //
        //      for i in range(iter):
        //          x = x - (x**rt - a)/(rt * x**lr)
        //
        //      return x

        let a = &self.fraction; // Original value, assigned to a

        let rt = BigFraction::new_raw(root.fraction.numer().unwrap().clone(), 1u8.into()); // The root we're trying to find, extracted from the numerator of the root argument

        let lroot = &rt - &BigFraction::one(); // Lesser root

        let mut x = a / &*TWO; // The value iteratively computed on

        let mut last_move = BigFraction::one(); // Used to find out if the last computation made a significant enough impact on the final value to justify performing another iteration of the algorithm

        // Will compute until every digit up to the precision is 100% accurate, in theory
        //
        while &last_move >= &min_move {
            // Keep iterating until the iterations result in inconsequential changes
            x = Self::round_denom(x, &prec_working); // Round the denominator to the working precision to make sure our BigFraction doesn't get too big...
            last_move = x.clone(); // Store our current x value into last_move to see the difference between the previous iteration and the current iteration
            x = &x - &(&(&Self::pow(&x, &rt) - a) / &(&rt * &Self::pow(&x, &lroot))); // x = x - (x**rt - a)/(rt * x**lr), in a beautiful reference mess
            last_move = (&x - &last_move).abs(); // Get the difference between the previous and current iteration
        }

        x = Self::round_denom(x, &prec_final);

        let result = Self {
            fraction: match root.fraction.is_negative() {
                true => BigFraction::one() / x,
                false => x,
            },
        };

        let exp = root.fraction.denom().unwrap().clone(); // Get the possible exponent we need to raise the rooted number to from the denominator of the root

        // If the root is a whole number, then we're done!
        if &exp == &BigUint::one() {
            result
        }
        // Otherwise raise the number to the power of the root's denominator
        else {
            let exp_num = Self {
                fraction: BigFraction::new_raw(exp, 1u8.into()),
            };

            Self::exponent(&result, &exp_num, prec)
        }
    }

    /// Rounds the denominator for quicker calculations which don't need perfect accuracy
    ///
    /// **PRIVATE FUNCTIONS**
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
