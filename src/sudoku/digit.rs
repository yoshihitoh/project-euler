use std::fmt;

use anyhow::{anyhow, Error as AnyhowError};
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fmt::Debug;

const MIN: u8 = 1;
const MAX: u8 = 9;

const RADIX: u32 = 10;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Digit(u8);

impl Digit {
    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn all_digits() -> BTreeSet<Digit> {
        (MIN..=MAX).map(Digit::from).collect()
    }

    pub fn all_digits_iter() -> DigitIter {
        DigitIter::default()
    }

    pub fn exclude_iter(digit: Digit) -> impl Iterator<Item = Digit> {
        DigitIter::default().filter(move |&d| d != digit)
    }
}

impl From<u8> for Digit {
    fn from(value: u8) -> Self {
        assert!(value >= MIN && value <= MAX);
        Digit(value)
    }
}

impl Into<char> for Digit {
    fn into(self) -> char {
        std::char::from_digit(self.0 as u32, RADIX).unwrap()
    }
}

impl TryFrom<char> for Digit {
    type Error = AnyhowError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let digit = value
            .to_digit(10)
            .ok_or_else(|| anyhow!("Cannot convert '{}' to Digit", value))?;
        Self::try_from(digit)
    }
}

impl TryFrom<u32> for Digit {
    type Error = AnyhowError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let digit = Some(u8::try_from(value)?)
            .filter(|&n| n >= MIN && n <= MAX)
            .ok_or_else(|| {
                anyhow!(
                    "out of range. must be within {} to {}, given:{}",
                    MIN,
                    MAX,
                    value
                )
            })?;
        Ok(Digit(digit))
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DigitIter {
    current: u8,
}

impl Iterator for DigitIter {
    type Item = Digit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < MAX {
            self.current += 1;
            Some(Digit::from(self.current))
        } else {
            None
        }
    }
}
