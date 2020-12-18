use crate::flags::Flags32;
use crate::sudoku::action::{ActionScope, RemoveAction, RetainAction};
use crate::sudoku::board::BoxPosition;
use crate::sudoku::candidate::Candidate;
use crate::sudoku::digit::Digit;
use crate::sudoku::event::{Event, EventQueue};
use crate::sudoku::filter::{
    FilterCandidates, FilterContext, FilterInput, NamedFilter, ScanCandidates,
};
use crate::sudoku::positions::{BoxPositions, ColumnPositions, Positions, RowPositions};
use crate::sudoku::set::DigitSet;
use crate::sudoku::{Board, Square};
use std::borrow::Borrow;
use std::iter::FromIterator;
use std::ops::Range;

fn collect_digit_positions<'a, 'b>(
    context: &'a mut FilterContext,
    candidates: impl Iterator<Item = &'b Candidate>,
) {
    context.digit_positions.clear();
    for (pos, c) in candidates.enumerate() {
        for d in Digit::all_digits_iter() {
            if c.contains(d) {
                context.digit_positions.entry(d).or_default().set(pos);
            }
        }
    }
}

fn collect_digit_positions_matches<'a, 'b, P>(
    context: &'a mut FilterContext,
    candidates: impl Iterator<Item = &'b Candidate>,
    predicate: P,
) where
    P: Fn(&Digit, &Positions) -> bool,
{
    collect_digit_positions(context, candidates);

    let mut remove_digits = DigitSet::default();
    for (digit, positions) in context.digit_positions.iter() {
        if !predicate(digit, positions) {
            remove_digits.set(*digit);
        }
    }

    for d in remove_digits {
        context.digit_positions.remove(&d);
    }
}

pub struct NakedSingle;

impl NakedSingle {
    fn remove_digits_in_use<'a>(
        &self,
        input: &mut FilterInput,
        scope: ActionScope,
        squares: impl Iterator<Item = &'a Square>,
    ) {
        input.context.digits_in_use.clear();
        input
            .context
            .digits_in_use
            .extend(squares.filter_map(|sq| sq.digit()));

        for d in input.context.digits_in_use.iter() {
            let event = Event::from(RemoveAction::new(d, scope));
            input.event_queue.push_back(event);
        }
    }
}

impl NamedFilter for NakedSingle {
    fn name(&self) -> &'static str {
        "NakedSingle"
    }
}

impl ScanCandidates for NakedSingle {
    fn scan_rows(&self, input: &mut FilterInput) {
        let positions = Positions::from_iter(input.board.each_rows());
        for row in input.board.each_rows() {
            self.remove_digits_in_use(
                input,
                ActionScope::Row(RowPositions::new(row, positions)),
                input.board.row_items(row),
            )
        }
    }

    fn scan_columns(&self, input: &mut FilterInput) {
        let positions = Positions::from_iter(input.board.each_columns());
        for col in input.board.each_columns() {
            self.remove_digits_in_use(
                input,
                ActionScope::Column(ColumnPositions::new(col, positions)),
                input.board.column_items(col),
            )
        }
    }

    fn scan_boxes(&self, input: &mut FilterInput) {
        let positions = Positions::from_iter(input.board.each_box_items());
        for box_pos in input.board.box_positions() {
            self.remove_digits_in_use(
                input,
                ActionScope::Box(BoxPositions::new(box_pos, positions)),
                input.board.box_at(box_pos),
            )
        }
    }
}

pub struct SingleCandidate;

impl SingleCandidate {
    fn scan_single_candidate<'a, F>(
        &self,
        context: &mut FilterContext,
        event_queue: &mut EventQueue,
        candidates: impl Iterator<Item = &'a Candidate>,
        scope_fn: F,
    ) where
        F: Fn(Positions) -> ActionScope,
    {
        // 各数字について、どの位置に候補が存在するかするかスキャンする
        collect_digit_positions_matches(context, candidates, |_, ps| ps.num_set() == 1);

        // 単一候補以外の数値を除外する。
        for (digit, positions) in context.digit_positions.iter() {
            let scope = scope_fn(*positions);
            let action = RetainAction::new(*digit, scope);
            let event = Event::from(action);
            event_queue.push_back(event);
        }
    }
}

impl NamedFilter for SingleCandidate {
    fn name(&self) -> &'static str {
        "SingleCandidate"
    }
}

impl ScanCandidates for SingleCandidate {
    fn scan_rows<'a>(&self, input: &mut FilterInput) {
        for row in input.board.each_rows() {
            self.scan_single_candidate(
                input.context,
                input.event_queue,
                input.candidates.row_items(row),
                |positions| ActionScope::Row(RowPositions::new(row, positions)),
            );
        }
    }

    fn scan_columns(&self, input: &mut FilterInput) {
        for col in input.board.each_columns() {
            self.scan_single_candidate(
                input.context,
                input.event_queue,
                input.candidates.column_items(col),
                |positions| ActionScope::Column(ColumnPositions::new(col, positions)),
            );
        }
    }

    fn scan_boxes(&self, input: &mut FilterInput) {
        for box_pos in input.board.box_positions() {
            self.scan_single_candidate(
                input.context,
                input.event_queue,
                input.candidates.box_at(box_pos),
                |positions| ActionScope::Box(BoxPositions::new(box_pos, positions)),
            );
        }
    }
}

pub struct LockedCandidate;

impl LockedCandidate {
    fn positions_exclude(box_offset: usize, box_size: usize, num_boxes: usize) -> Positions {
        let item_start = box_offset * box_size;
        let excludes = item_start..(item_start + box_size);
        let iter = (0..(box_size * num_boxes)).filter(|i| !excludes.contains(i));
        Positions::from_iter(iter)
    }

    fn scan_pointing<'a, FI, I, FS>(
        &self,
        context: &mut FilterContext,
        event_queue: &mut EventQueue,
        box_offsets: Range<usize>,
        box_candidates: impl Iterator<Item = &'a Candidate>,
        iter_fn: FI,
        scope_fn: FS,
    ) where
        FI: Fn(usize) -> I,
        I: Iterator<Item = &'a Candidate>,
        FS: Fn(usize) -> ActionScope,
    {
        collect_digit_positions_matches(context, box_candidates, |digit, positions| {
            positions.num_set() >= 2 || positions.num_set() <= 3
        });

        for (digit, positions) in context.digit_positions.iter() {
            for offset in box_offsets.clone() {
                let num_digits = iter_fn(offset).filter(|c| c.contains(*digit)).count();
                if num_digits == positions.num_set() {
                    // ボックスのどちらかに入るので、その他のボックスの列・行から除外する
                    let scope = scope_fn(offset);
                    let action = RemoveAction::new(*digit, scope);
                    let event = Event::from(action);
                    event_queue.push_back(event);
                }
            }
        }
    }

    //
    // fn claiming(&self, _input: &FilterInput) -> bool {
    //     false
    // }
}

impl NamedFilter for LockedCandidate {
    fn name(&self) -> &'static str {
        "LockedCandidate"
    }
}

impl ScanCandidates for LockedCandidate {
    fn scan_rows(&self, input: &mut FilterInput) {
        let box_size = input.board.box_size();
        let num_boxes = input.board.num_boxes();
        for box_pos in input.board.box_positions() {
            let candidates = input.candidates;
            self.scan_pointing(
                input.context,
                input.event_queue,
                input.board.each_box_rows(),
                input.candidates.box_at(box_pos),
                |box_row| candidates.box_at(box_pos).row_items(box_row),
                |box_row| {
                    let row = box_pos.row * box_size + box_row;
                    let positions = Self::positions_exclude(box_pos.col, box_size, num_boxes);
                    ActionScope::Row(RowPositions::new(row, positions))
                },
            );
        }
    }

    fn scan_columns(&self, input: &mut FilterInput) {
        let box_size = input.board.box_size();
        let num_boxes = input.board.num_boxes();
        for box_pos in input.board.box_positions() {
            let candidates = input.candidates;
            self.scan_pointing(
                input.context,
                input.event_queue,
                input.board.each_box_columns(),
                input.candidates.box_at(box_pos),
                |box_col| candidates.box_at(box_pos).column_items(box_col),
                |box_col| {
                    let col = box_pos.col * box_size + box_col;
                    let positions = Self::positions_exclude(box_pos.row, box_size, num_boxes);
                    ActionScope::Column(ColumnPositions::new(col, positions))
                },
            );
        }
    }

    fn scan_boxes(&self, _input: &mut FilterInput) {
        //
    }
}
