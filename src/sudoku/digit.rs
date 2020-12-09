use std::fmt;

use anyhow::{anyhow, Error as AnyhowError};
use std::collections::BTreeSet;
use std::convert::TryFrom;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Digit(u8);

impl Digit {
    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn all_digits() -> BTreeSet<Digit> {
        (1..=9).map(Digit::from).collect()
    }

    pub fn all_digits_iter() -> impl Iterator<Item = Digit> {
        (1..=9).map(Digit::from)
    }
}

impl From<u8> for Digit {
    fn from(value: u8) -> Self {
        Digit(value)
    }
}

impl Into<char> for Digit {
    fn into(self) -> char {
        std::char::from_digit(self.0 as u32, 10).unwrap()
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
        let digit = Some(value)
            .filter(|&n| n != 0 && n < 10)
            .ok_or_else(|| anyhow!("Cannot convert '{}' to Digit", value))?;
        Ok(Digit(digit as u8))
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
