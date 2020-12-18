use std::convert::TryFrom;
use std::fmt;

use crate::sudoku::digit::Digit;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Square {
    digit: Option<Digit>,
    row: usize,
    column: usize,
}

impl Square {
    pub fn new(digit: Option<Digit>, row: usize, column: usize) -> Self {
        Square { digit, row, column }
    }

    pub fn digit(&self) -> Option<Digit> {
        self.digit
    }

    pub fn is_fixed(&self) -> bool {
        self.digit.is_some()
    }

    pub fn fix_digit(&mut self, digit: Digit) {
        assert!(self.digit.is_none());
        self.digit = Some(digit);
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = self
            .digit
            .and_then(|d| std::char::from_digit(d.get() as u32, 10))
            .unwrap_or('-');
        fmt::Display::fmt(&c, f)
    }
}
