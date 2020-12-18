use crate::sudoku::candidate::Candidate;
use crate::sudoku::digit::Digit;
use crate::sudoku::positions::{BoxPositions, ColumnPositions, Positions, RowPositions};
use crate::sudoku::Board;
use itertools::Itertools;
use std::borrow::Borrow;

#[derive(Debug, Copy, Clone)]
pub enum ActionScope {
    Row(RowPositions),
    Column(ColumnPositions),
    Box(BoxPositions),
}

#[derive(Debug, Copy, Clone)]
pub struct RetainAction {
    digit: Digit,
    scope: ActionScope,
}

impl RetainAction {
    pub fn new(digit: Digit, scope: ActionScope) -> Self {
        RetainAction { digit, scope }
    }

    pub fn retain(self, candidates: &mut Board<Candidate>) -> bool {
        match self.scope {
            ActionScope::Row(pos) => {
                self.retain_iter(pos.columns, candidates.row_items_mut(pos.row))
            }
            ActionScope::Column(pos) => {
                self.retain_iter(pos.rows, candidates.column_items_mut(pos.column))
            }
            ActionScope::Box(pos) => {
                self.retain_iter(pos.indexes, candidates.box_at_mut(pos.box_at))
            }
        }
    }

    fn retain_iter<'a, I>(self, positions: Positions, iter: I) -> bool
    where
        I: Iterator<Item = &'a mut Candidate>,
    {
        let digits_to_remove = Digit::all_digits_iter()
            .filter(|d| *d != self.digit)
            .map(|d| format!("{}", d))
            .collect_vec();
        positions.items_from_iter(iter).fold(false, |updated, c| {
            c.remove_iter(Digit::all_digits_iter().filter(|d| *d != self.digit)) || updated
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
            ActionScope::Box(pos) => {
                self.remove_iter(pos.indexes, candidates.box_at_mut(pos.box_at))
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
