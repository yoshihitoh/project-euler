use std::fmt::{self, Debug};

use itertools::Itertools;
use thiserror::Error;

use crate::sudoku::candidate::Candidate;
use crate::sudoku::digit::Digit;
use crate::sudoku::printer::{BoardPrinter, Printer};
use crate::sudoku::square::Square;
use std::convert::TryFrom;
use std::ops::Range;

fn enumerate_table_positions(
    num_rows: usize,
    num_cols: usize,
) -> impl Iterator<Item = (usize, usize)> {
    (0..num_rows).flat_map(move |row| (0..num_cols).map(move |col| (row, col)))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Scope {
    Row(usize),
    Column(usize),
    BoardBox(BoxPosition),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ItemPosition {
    pub row: usize,
    pub col: usize,
}

impl fmt::Display for ItemPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item({}, {})", self.row, self.col)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BoxPosition {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("duplication error. digit:{0:?}, scope:{1:?}")]
    Duplication(Scope, Vec<Digit>),
}

#[derive(Debug, Clone)]
pub struct Board<T: Debug + Clone> {
    items: Vec<T>,
    box_size: usize,
    num_boxes: usize,
}

impl<T: Debug + Clone> Board<T> {
    pub fn new(items: Vec<T>, box_size: usize, num_boxes: usize) -> Self {
        Board {
            items,
            box_size,
            num_boxes,
        }
    }

    pub fn box_positions(&self) -> impl Iterator<Item = BoxPosition> {
        enumerate_table_positions(self.num_boxes(), self.num_boxes()).map(|(box_row, box_col)| {
            BoxPosition {
                row: box_row,
                col: box_col,
            }
        })
    }

    pub fn box_size(&self) -> usize {
        self.box_size
    }

    pub fn each_box_columns(&self) -> Range<usize> {
        0..self.box_size
    }

    pub fn each_box_rows(&self) -> Range<usize> {
        0..self.box_size
    }

    pub fn num_boxes(&self) -> usize {
        self.num_boxes
    }

    pub fn each_box_items(&self) -> Range<usize> {
        0..(self.box_size * self.num_boxes)
    }

    pub fn box_at(&self, pos: BoxPosition) -> BoardBox<impl Iterator<Item = &T>> {
        assert!(pos.row < self.num_boxes);
        assert!(pos.col < self.num_boxes);

        let row = pos.row * self.num_boxes;
        let col = pos.col * self.num_boxes;

        let iter = self.each_box_rows().flat_map(move |row_offset| {
            let index = self.index_of(row + row_offset, col);
            self.items.iter().skip(index).take(self.box_size)
        });
        BoardBox {
            iter,
            box_size: self.box_size,
        }
    }

    pub fn item_positions(&self) -> impl Iterator<Item = ItemPosition> {
        enumerate_table_positions(self.height(), self.width())
            .map(|(row, col)| ItemPosition { row, col })
    }

    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn item_at(&self, pos: ItemPosition) -> &T {
        let index = self.index_of(pos.row, pos.col);
        &self.items[index]
    }

    pub fn width(&self) -> usize {
        self.box_size * self.num_boxes
    }

    pub fn each_columns(&self) -> Range<usize> {
        0..self.width()
    }

    pub fn column_items(&self, col: usize) -> impl Iterator<Item = &T> {
        let index = self.index_of(0, col);
        self.items.iter().skip(index).step_by(self.width())
    }

    pub fn height(&self) -> usize {
        self.box_size * self.num_boxes
    }

    pub fn each_rows(&self) -> Range<usize> {
        0..self.height()
    }

    pub fn row_items(&self, row: usize) -> impl Iterator<Item = &T> {
        let index = self.index_of(row, 0);
        self.items.iter().skip(index).take(self.width())
    }

    fn index_of(&self, row: usize, col: usize) -> usize {
        row * self.width() + col
    }
}

impl Board<Square> {
    pub fn fix_digit_at(&mut self, pos: ItemPosition, digit: Digit) -> Result<(), BoardError> {
        DuplicationValidator::new(self).validate(pos, Some(digit))?;

        let index = self.index_of(pos.row, pos.col);
        self.items[index].fix_digit(digit);

        Ok(())
    }

    pub fn show(&self) {
        BoardPrinter::new(self).show();
    }

    pub fn is_complete(&self) -> bool {
        self.items.iter().all(|sq| sq.is_fixed())
    }

    pub fn validate(&self) -> Result<(), BoardError> {
        let mut validator = DuplicationValidator::new(self);

        for row in self.each_rows() {
            validator.validate_with_scope(Scope::Row(row), None)?;
        }

        for col in self.each_columns() {
            validator.validate_with_scope(Scope::Column(col), None)?;
        }

        for box_pos in self.box_positions() {
            validator.validate_with_scope(Scope::BoardBox(box_pos), None)?;
        }

        Ok(())
    }
}

impl Board<Candidate> {
    pub fn take_fixed_digit_at(&mut self, pos: ItemPosition) -> Option<Digit> {
        let index = self.index_of(pos.row, pos.col);
        self.items[index].take_fixed_digit()
    }

    pub fn row_items_mut(&mut self, row: usize) -> impl Iterator<Item = &mut Candidate> {
        let index = self.index_of(row, 0);
        let width = self.width();
        self.items.iter_mut().skip(index).take(width)
    }

    pub fn column_items_mut(&mut self, col: usize) -> impl Iterator<Item = &mut Candidate> {
        let index = self.index_of(0, col);
        let width = self.width();
        self.items.iter_mut().skip(index).step_by(width)
    }

    pub fn box_at_mut(
        &mut self,
        pos: BoxPosition,
    ) -> BoardBox<impl Iterator<Item = &mut Candidate>> {
        assert!(pos.row < self.num_boxes);
        assert!(pos.col < self.num_boxes);

        let row = pos.row * self.num_boxes;
        let col = pos.col * self.num_boxes;

        let left_top_index = self.index_of(row, col);
        let width = self.width();
        let height = self.height();
        let box_size = self.box_size;

        let iter = self
            .items
            .iter_mut()
            .enumerate()
            .skip(left_top_index)
            // TODO: takeで終了条件を追加すること、現状はボックス範囲外を抜けてもループしてる
            .filter(move |(index, _)| {
                let r = index / height;
                let c = index % width;

                let row_matches = r >= row && r < row + box_size;
                let col_matches = c >= col && c < col + box_size;

                row_matches && col_matches
            })
            .map(|(_, square)| square);

        BoardBox {
            iter,
            box_size: self.box_size,
        }
    }

    pub fn show(&self) {
        BoardPrinter::new(self).show();
    }
}

pub struct BoardBox<I> {
    iter: I,
    box_size: usize,
}

impl<I: Iterator> BoardBox<I> {
    pub fn row_items(self, box_row: usize) -> impl Iterator<Item = I::Item> {
        let offset = box_row * self.box_size;
        self.iter.skip(offset).take(self.box_size)
    }

    pub fn column_items(self, box_col: usize) -> impl Iterator<Item = I::Item> {
        self.iter
            .skip(box_col)
            .step_by(self.box_size)
            .take(self.box_size)
    }
}

impl<I: Iterator> Iterator for BoardBox<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct BoardLoader {}

impl BoardLoader {
    pub fn from_lines<'a>(lines: impl Iterator<Item = &'a str>) -> Board<Square> {
        let lines = lines.collect_vec();
        let height = lines.len();
        let width = height;

        let box_size = (height as f64).sqrt() as usize;
        if height % box_size != 0 {
            panic!(
                "wrong input. width={}, height={}, box_size={}",
                width, height, box_size
            );
        }
        let items = lines
            .into_iter()
            .enumerate()
            .flat_map(|(row, s)| {
                if s.len() != width {
                    panic!(
                        "Wrong line, must contains {} chars, found {} chars on '{}'",
                        width,
                        s.len(),
                        s
                    );
                }
                s.chars().enumerate().map(move |(col, c)| {
                    let d = Digit::try_from(c).ok();
                    Square::new(d, row, col)
                })
            })
            .collect::<Vec<_>>();

        if items.len() != width * height {
            panic!("Wrong board size");
        }

        let num_boxes = height / box_size;
        Board::new(items, box_size, num_boxes)
    }
}

struct DuplicationValidator<'a> {
    board: &'a Board<Square>,
}

impl<'a> DuplicationValidator<'a> {
    pub fn new(board: &'a Board<Square>) -> Self {
        DuplicationValidator { board }
    }

    pub fn validate(
        &mut self,
        item_pos: ItemPosition,
        digit: Option<Digit>,
    ) -> Result<(), BoardError> {
        self.validate_with_scope(Scope::Row(item_pos.row), digit)?;
        self.validate_with_scope(Scope::Column(item_pos.col), digit)?;

        let box_pos = BoxPosition {
            row: item_pos.row / self.board.num_boxes(),
            col: item_pos.col / self.board.num_boxes(),
        };
        self.validate_with_scope(Scope::BoardBox(box_pos), digit)?;

        Ok(())
    }

    pub fn validate_with_scope(
        &mut self,
        scope: Scope,
        digit: Option<Digit>,
    ) -> Result<(), BoardError> {
        match scope {
            Scope::Row(row) => self.validate_with_iter(scope, digit, self.board.row_items(row)),
            Scope::Column(col) => {
                self.validate_with_iter(scope, digit, self.board.column_items(col))
            }
            Scope::BoardBox(pos) => self.validate_with_iter(scope, digit, self.board.box_at(pos)),
        }
    }

    fn validate_with_iter<I>(
        &mut self,
        scope: Scope,
        digit: Option<Digit>,
        iter: I,
    ) -> Result<(), BoardError>
    where
        I: Iterator<Item = &'a Square>,
    {
        let mut validated = 0;
        let mut validate = |d: Digit| {
            let mask = 0x01 << (d.get() - 1) as u32;
            let is_available = validated & mask == 0;
            if is_available {
                validated |= mask;
                true
            } else {
                false
            }
        };

        let mut duplications = Vec::new();
        for d in iter.flat_map(|sq| sq.digit()) {
            if !validate(d) {
                duplications.push(d);
            }
        }

        if let Some(d) = digit {
            if !validate(d) {
                duplications.push(d);
            }
        }

        if duplications.is_empty() {
            Ok(())
        } else {
            dbg!((scope, &duplications));
            Err(BoardError::Duplication(scope, duplications))
        }
    }
}
