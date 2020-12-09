use std::fmt;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitOr};

use crate::flags::Flags32;
use crate::sudoku::digit::{Digit, DigitIter};
use itertools::Itertools;

fn digit_position(d: Digit) -> usize {
    d.get() as usize - 1
}

#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct DigitSet {
    flags: Flags32,
}

impl DigitSet {
    pub fn contains(&self, d: Digit) -> bool {
        self.flags.get(digit_position(d))
    }

    pub fn is_empty(&self) -> bool {
        self.flags.num_set() == 0
    }

    pub fn len(&self) -> usize {
        self.flags.num_set()
    }

    pub fn clear(&mut self) {
        self.flags.clear();
    }

    pub fn set(&mut self, d: Digit) {
        self.flags.set(digit_position(d));
    }

    pub fn extend<I: Iterator<Item = Digit>>(&mut self, iter: I) {
        iter.for_each(|d| self.set(d));
    }

    pub fn remove(&mut self, d: Digit) -> bool {
        if self.contains(d) {
            self.flags.unset(digit_position(d));
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> DigitSetIter {
        DigitSetIter::new(*self)
    }
}

impl fmt::Debug for DigitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DigitSet({})",
            self.iter()
                .map(|d| format!("{}", d))
                .collect_vec()
                .join(", ")
        )
    }
}

impl IntoIterator for DigitSet {
    type Item = Digit;
    type IntoIter = DigitSetIter;

    fn into_iter(self) -> Self::IntoIter {
        DigitSetIter::new(self)
    }
}

impl FromIterator<Digit> for DigitSet {
    fn from_iter<T: IntoIterator<Item = Digit>>(iter: T) -> Self {
        let mut s = DigitSet::default();
        s.extend(iter.into_iter());
        s
    }
}

impl BitAnd for DigitSet {
    type Output = DigitSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        DigitSet {
            flags: self.flags & rhs.flags,
        }
    }
}

impl BitOr for DigitSet {
    type Output = DigitSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        DigitSet {
            flags: self.flags | rhs.flags,
        }
    }
}

pub struct DigitSetIter {
    set: DigitSet,
    iter: DigitIter,
}

impl DigitSetIter {
    pub fn new(set: DigitSet) -> Self {
        let iter = Digit::all_digits_iter();
        DigitSetIter { set, iter }
    }
}

impl Iterator for DigitSetIter {
    type Item = Digit;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(d) = self.iter.next() {
            if self.set.contains(d) {
                return Some(d);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::sudoku::digit::Digit;
    use crate::sudoku::set::DigitSet;

    #[test]
    fn test_empty() {
        let s = DigitSet::default();
        for i in 1..=9 {
            assert!(!s.contains(Digit::from(i)))
        }
    }

    #[test]
    fn test_isolation() {
        let mut s = DigitSet::default();
        for target in 1..=9 {
            s.clear();
            assert!(!s.contains(Digit::from(target)));
            s.set(Digit::from(target));

            for i in 1..=9 {
                if i == target {
                    assert!(s.contains(Digit::from(i)));
                } else {
                    assert!(!s.contains(Digit::from(i)));
                }
            }
        }
    }

    #[test]
    fn test_all() {
        let mut s = DigitSet::default();
        for i in 1..=9 {
            s.set(Digit::from(i));
        }

        for i in 1..=9 {
            assert!(s.contains(Digit::from(i)));
        }
    }
}
