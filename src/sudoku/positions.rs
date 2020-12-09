use std::fmt;
use std::iter::FromIterator;

use crate::flags::Flags32;
use crate::sudoku::board::BlockPosition;

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct Positions(Flags32);

impl fmt::Debug for Positions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indexes = self.iter().map(|i| format!("{}", i)).collect::<Vec<_>>();
        write!(f, "Positions({})", indexes.join(", "))
    }
}

impl Positions {
    pub fn with_positions<I: Iterator<Item = usize>>(iter: I) -> Positions {
        let mut flags = Flags32::default();
        for i in iter {
            flags.set(i);
        }

        Positions(flags)
    }

    pub fn with_offset(offset: usize, num: usize) -> Positions {
        Self::with_positions(offset..(offset + num))
    }

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

    pub fn belongs_to(&self, other: &Positions) -> bool {
        self.0 & other.0 == self.0
    }

    pub fn and(&self, other: &Positions) -> Positions {
        Positions(self.0 & other.0)
    }

    pub fn invert(&self, max_length: usize) -> Positions {
        let mask = (0x01 << max_length) - 1;
        Positions(!self.0 & Flags32::from(mask))
    }
}

impl FromIterator<usize> for Positions {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        Positions::with_positions(iter.into_iter())
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
pub struct BlockPositions {
    pub block_at: BlockPosition,
    pub indexes: Positions,
}

impl BlockPositions {
    pub fn new(block_at: BlockPosition, positions: Positions) -> Self {
        BlockPositions {
            block_at,
            indexes: positions,
        }
    }
}
