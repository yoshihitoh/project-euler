use std::collections::BTreeMap;

use crate::sudoku::candidate::Candidate;
use crate::sudoku::digit::Digit;
use crate::sudoku::set::DigitSet;
use crate::sudoku::{Board, Square};

mod basic;

use crate::sudoku::event::EventQueue;
use crate::sudoku::positions::Positions;
pub use basic::LockedCandidate;
pub use basic::NakedSingle;
pub use basic::SingleCandidate;

#[derive(Default)]
pub struct FilterContext {
    digits_in_use: DigitSet,
    digit_positions: BTreeMap<Digit, Positions>,
}

pub struct FilterInput<'a> {
    context: &'a mut FilterContext,
    event_queue: &'a mut EventQueue,
    board: &'a Board<Square>,
    candidates: &'a Board<Candidate>,
}

impl<'a> FilterInput<'a> {
    pub fn new(
        context: &'a mut FilterContext,
        event_queue: &'a mut EventQueue,
        board: &'a Board<Square>,
        candidates: &'a Board<Candidate>,
    ) -> Self {
        FilterInput {
            context,
            event_queue,
            board,
            candidates,
        }
    }
}

pub trait NamedFilter {
    fn name(&self) -> &'static str;
}

pub trait FilterCandidates: NamedFilter {
    fn filter_candidates(&self, input: FilterInput);
}

pub trait ScanCandidates: FilterCandidates {
    fn scan_rows(&self, input: &mut FilterInput);
    fn scan_columns(&self, input: &mut FilterInput);
    fn scan_boxes(&self, input: &mut FilterInput);
}

impl<T: ScanCandidates> FilterCandidates for T {
    fn filter_candidates(&self, mut input: FilterInput) {
        self.scan_rows(&mut input);
        self.scan_columns(&mut input);
        self.scan_boxes(&mut input);
    }
}
