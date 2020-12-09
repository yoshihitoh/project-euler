use crate::sudoku::candidate::Candidate;
use crate::sudoku::digit::Digit;
use crate::sudoku::positions::{BlockPositions, ColumnPositions, Positions, RowPositions};
use crate::sudoku::set::DigitSet;
use crate::sudoku::Board;

#[derive(Debug, Copy, Clone)]
pub enum ActionScope {
    Row(RowPositions),
    Column(ColumnPositions),
    Block(BlockPositions),
}

#[derive(Debug, Copy, Clone)]
pub struct RetainAction {
    digits: DigitSet,
    scope: ActionScope,
}

impl RetainAction {
    pub fn new(digits: DigitSet, scope: ActionScope) -> Self {
        RetainAction { digits, scope }
    }

    pub fn with_digit(digit: Digit, scope: ActionScope) -> Self {
        let mut digits = DigitSet::default();
        digits.set(digit);

        Self::new(digits, scope)
    }

    pub fn retain(self, candidates: &mut Board<Candidate>) -> bool {
        match self.scope {
            ActionScope::Row(pos) => {
                self.retain_iter(pos.columns, candidates.row_items_mut(pos.row))
            }
            ActionScope::Column(pos) => {
                self.retain_iter(pos.rows, candidates.column_items_mut(pos.column))
            }
            ActionScope::Block(pos) => {
                self.retain_iter(pos.indexes, candidates.block_at_mut(pos.block_at))
            }
        }
    }

    fn retain_iter<'a, I>(self, positions: Positions, iter: I) -> bool
    where
        I: Iterator<Item = &'a mut Candidate>,
    {
        positions.items_from_iter(iter).fold(false, |updated, c| {
            c.remove_iter(Digit::all_digits_iter().filter(|d| !self.digits.contains(*d))) || updated
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RemoveAction {
    digit: Digit,
    scope: ActionScope,
}

impl RemoveAction {
    pub fn new(digit: Digit, scope: ActionScope) -> Self {
        RemoveAction { digit, scope }
    }

    pub fn remove(self, candidates: &mut Board<Candidate>) -> bool {
        match self.scope {
            ActionScope::Row(pos) => {
                self.remove_iter(pos.columns, candidates.row_items_mut(pos.row))
            }
            ActionScope::Column(pos) => {
                self.remove_iter(pos.rows, candidates.column_items_mut(pos.column))
            }
            ActionScope::Block(pos) => {
                self.remove_iter(pos.indexes, candidates.block_at_mut(pos.block_at))
            }
        }
    }

    fn remove_iter<'a, I>(self, positions: Positions, iter: I) -> bool
    where
        I: Iterator<Item = &'a mut Candidate>,
    {
        positions
            .items_from_iter(iter)
            .fold(false, |updated, c| c.remove(self.digit) || updated)
    }
}
