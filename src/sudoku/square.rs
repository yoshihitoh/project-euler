use std::collections::BTreeSet;
use std::fmt;
use std::iter::FromIterator;

use crate::sudoku::digit::Digit;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Square {
    digit: Option<Digit>,
    candidates: BTreeSet<Digit>,
}

impl Square {
    pub fn new(digit: Option<Digit>) -> Self {
        let candidates = digit
            .as_ref()
            .map(|_| BTreeSet::new())
            .unwrap_or(BTreeSet::from_iter(Digit::all_digits_iter()));

        Square { digit, candidates }
    }

    pub fn digit(&self) -> Option<Digit> {
        self.digit
    }

    pub fn is_fixed(&self) -> bool {
        self.digit.is_some()
    }

    pub fn fix_digit(&mut self, digit: Digit) {
        self.digit = Some(digit);
        self.candidates.clear();
    }

    pub fn candidates(&self) -> &BTreeSet<Digit> {
        &self.candidates
    }

    pub fn remove_candidate(&mut self, digit: Digit) -> bool {
        self.digit.map(|_| false).unwrap_or_else(|| {
            let removed = self.candidates.remove(&digit);
            if self.candidates.len() == 1 {
                // TODO: unstable外れたらpop_first()にする
                self.digit = self.candidates.iter().next().copied();
                self.candidates.clear();
            }
            removed
        })
    }

    pub fn remove_candidates_iter(&mut self, digits: impl Iterator<Item = Digit>) -> bool {
        digits.fold(false, |updated, d| self.remove_candidate(d) || updated)
    }
}

impl Default for Square {
    fn default() -> Self {
        Square {
            digit: None,
            candidates: BTreeSet::from_iter(Digit::all_digits_iter()),
        }
    }
}

impl From<char> for Square {
    fn from(c: char) -> Self {
        Square::new(
            c.to_digit(10)
                .filter(|&d| d != 0)
                .map(|d| d as u8)
                .map(Digit::from),
        )
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = self
            .digit
            .and_then(|d| std::char::from_digit(d.get() as u32, 10))
            .unwrap_or('-');
        write!(f, "{}", c)
    }
}
