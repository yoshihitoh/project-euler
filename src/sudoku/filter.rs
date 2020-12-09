mod hidden;
mod intersection;
mod single;

use std::collections::BTreeMap;

use crate::sudoku::candidate::Candidate;
use crate::sudoku::digit::Digit;
use crate::sudoku::event::EventQueue;
use crate::sudoku::positions::Positions;
use crate::sudoku::set::DigitSet;
use crate::sudoku::{Board, Square};

pub use hidden::{HiddenPair, HiddenQuad, HiddenTriple};
pub use intersection::{LockedCandidateClaiming, LockedCandidatePointing};
pub use single::{NakedSingle, SingleCandidate};

#[derive(Default)]
pub struct FilterContext {
    digits_in_use: DigitSet,
    digit_positions: BTreeMap<Digit, Positions>,
}

impl FilterContext {
    pub fn collect_digit_positions<'a, 'b>(
        self: &'a mut FilterContext,
        candidates: impl Iterator<Item = &'b Candidate>,
    ) {
        self.digit_positions.clear();
        for (pos, c) in candidates.enumerate() {
            for d in Digit::all_digits_iter() {
                if c.contains(d) {
                    self.digit_positions.entry(d).or_default().set(pos);
                }
            }
        }
    }

    pub fn collect_digit_positions_matches<'a, 'b, P>(
        self: &'a mut FilterContext,
        candidates: impl Iterator<Item = &'b Candidate>,
        predicate: P,
    ) where
        P: Fn(&Digit, &Positions) -> bool,
    {
        self.collect_digit_positions(candidates);

        let mut remove_digits = DigitSet::default();
        for (digit, positions) in self.digit_positions.iter() {
            if !predicate(digit, positions) {
                remove_digits.set(*digit);
            }
        }

        for d in remove_digits {
            self.digit_positions.remove(&d);
        }
    }
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
    fn scan_blocks(&self, input: &mut FilterInput);
}

impl<T: ScanCandidates> FilterCandidates for T {
    fn filter_candidates(&self, mut input: FilterInput) {
        self.scan_rows(&mut input);
        self.scan_columns(&mut input);
        self.scan_blocks(&mut input);
    }
}
