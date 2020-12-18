use crate::sudoku::board::ItemPosition;
use crate::sudoku::candidate::Candidate;
use crate::sudoku::{Board, Square};
use std::fmt::Debug;

pub trait Printer {
    fn show(&self);
}

pub struct BoardPrinter<'a, T: Debug + Clone> {
    board: &'a Board<T>,
}

impl<'a, T: Debug + Clone> BoardPrinter<'a, T> {
    pub fn new(board: &'a Board<T>) -> Self {
        BoardPrinter { board }
    }
}

impl<'a> Printer for BoardPrinter<'a, Square> {
    fn show(&self) {
        let show_separator = || {
            for col in self.board.each_columns() {
                if col % self.board.box_size() == 0 {
                    print!("+-");
                }
                print!("--");
            }
            println!("+");
        };

        for pos in self.board.item_positions() {
            if pos.row % self.board.box_size() == 0 && pos.col == 0 {
                show_separator();
            }

            if pos.col % self.board.box_size() == 0 {
                print!("| ");
            }

            print!("{:2}", self.board.item_at(pos));
            if pos.col + 1 == self.board.width() {
                println!("|");
            }
        }
        show_separator();
    }
}

impl<'a> Printer for BoardPrinter<'a, Candidate> {
    fn show(&self) {
        let show_separator = || {
            for _ in self.board.each_columns() {
                print!("+-");
                for _ in self.board.each_box_columns() {
                    print!("--");
                }
            }
            println!("+");
        };

        for row in self.board.each_rows() {
            show_separator();
            for box_row in self.board.each_box_rows() {
                for col in self.board.each_columns() {
                    let candidates = self.board.item_at(ItemPosition { row, col });
                    print!("|");
                    let iter = candidates
                        .possible_digits()
                        .skip(box_row * self.board.box_size())
                        .map(|d| Some(d))
                        .chain((0..).map(|_| None))
                        .take(self.board.box_size());
                    for d in iter {
                        if let Some(d) = d {
                            print!("{:2}", d);
                        } else {
                            print!("  ");
                        }
                    }
                    print!(" ");
                }
                println!("|");
            }
        }

        show_separator();
    }
}
