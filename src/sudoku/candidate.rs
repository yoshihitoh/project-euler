use std::fmt;
use std::iter::FromIterator;

use crate::sudoku::digit::Digit;
use crate::sudoku::set::DigitSet;
use crate::sudoku::Square;

#[derive(Clone)]
pub struct Candidate {
    digits: DigitSet,
    row: usize,
    column: usize,
}

impl Candidate {
    pub fn new(sq: &Square) -> Self {
        let digits = if sq.digit().is_some() {
            DigitSet::default()
        } else {
            DigitSet::from_iter(Digit::all_digits_iter())
        };
        Candidate {
            digits,
            row: sq.row(),
            column: sq.column(),
        }
    }

    pub fn remove(&mut self, digit: Digit) -> bool {
        self.digits.remove(digit)
    }

    pub fn remove_iter(&mut self, digits: impl Iterator<Item = Digit>) -> bool {
        digits.fold(false, |updated, d| self.remove(d) || updated)
    }

    pub fn has_candidate(&self) -> bool {
        !self.digits.is_empty()
    }

    pub fn contains(&self, d: Digit) -> bool {
        self.digits.contains(d)
    }

    pub fn is_fixed(&self) -> bool {
        if self.digits.len() == 1 {
            true
        } else {
            false
        }
    }

    pub fn take_fixed_digit(&mut self) -> Option<Digit> {
        if self.digits.len() == 1 {
            let fixed_digit = self.digits.iter().next();
            self.digits.clear();
            fixed_digit
        } else {
            None
        }
    }

    pub fn possible_digits(&self) -> impl Iterator<Item = Digit> + '_ {
        self.digits.iter()
    }

    pub fn digits_vec(&self) -> Vec<Digit> {
        self.digits_iter().collect()
    }

    pub fn digits(&self) -> DigitSet {
        self.digits
    }

    pub fn digits_iter(&self) -> impl Iterator<Item = Digit> + '_ {
        self.digits.iter()
    }
}

impl fmt::Debug for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = if self.is_fixed() {
            "Fixed"
        } else {
            "Candidate"
        };

        let digits = self
            .digits_iter()
            .map(|d| format!("{}", d))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}({})", label, digits)
    }
}
