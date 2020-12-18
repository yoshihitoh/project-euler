use std::fmt;
use std::iter::FromIterator;

use crate::flags::Flags32;
use crate::sudoku::board::BoxPosition;

#[derive(Copy, Clone, Default)]
pub struct Positions(Flags32);

impl fmt::Debug for Positions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indexes = self.iter().map(|i| format!("{}", i)).collect::<Vec<_>>();
        write!(f, "Positions({})", indexes.join(", "))
    }
}

impl Positions {
    pub fn set(&mut self, pos: usize) {
        self.0.set(pos);
    }

    pub fn items_from_iter<T>(self, iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
        iter.enumerate()
            .filter(move |(i, _)| self.0.get(*i))
            .map(|(_, x)| x)
    }

    pub fn num_set(&self) -> usize {
        self.0.num_set()
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, b)| *b)
            .map(|(i, _)| i)
    }

    pub fn contains(&self, pos: usize) -> bool {
        self.0.get(pos)
    }
}

impl FromIterator<usize> for Positions {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut flags = Flags32::default();
        for i in iter {
            flags.set(i);
        }

        Positions(flags)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RowPositions {
    pub row: usize,
    pub columns: Positions,
}

impl RowPositions {
    pub fn new(row: usize, positions: Positions) -> Self {
        RowPositions {
            row,
            columns: positions,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ColumnPositions {
    pub column: usize,
    pub rows: Positions,
}

impl ColumnPositions {
    pub fn new(column: usize, positions: Positions) -> Self {
        ColumnPositions {
            column,
            rows: positions,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BoxPositions {
    pub box_at: BoxPosition,
    pub indexes: Positions,
}

impl BoxPositions {
    pub fn new(box_at: BoxPosition, positions: Positions) -> Self {
        BoxPositions {
            box_at,
            indexes: positions,
        }
    }
}
