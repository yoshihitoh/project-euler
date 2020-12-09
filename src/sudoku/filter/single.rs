use std::iter::FromIterator;

use crate::sudoku::action::{ActionScope, RemoveAction, RetainAction};
use crate::sudoku::candidate::Candidate;
use crate::sudoku::event::{Event, EventQueue};
use crate::sudoku::filter::{FilterContext, FilterInput, NamedFilter, ScanCandidates};
use crate::sudoku::positions::{BlockPositions, ColumnPositions, Positions, RowPositions};
use crate::sudoku::Square;

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

    fn scan_blocks(&self, input: &mut FilterInput) {
        let positions = Positions::from_iter(input.board.block_item_indexes());
        for block_pos in input.board.block_positions() {
            self.remove_digits_in_use(
                input,
                ActionScope::Block(BlockPositions::new(block_pos, positions)),
                input.board.block_at(block_pos),
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
        context.collect_digit_positions_matches(candidates, |_, ps| ps.num_set() == 1);

        // 単一候補以外の数値を除外する。
        for (digit, positions) in context.digit_positions.iter() {
            let scope = scope_fn(*positions);
            let action = RetainAction::with_digit(*digit, scope);
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

    fn scan_blocks(&self, input: &mut FilterInput) {
        for block_pos in input.board.block_positions() {
            self.scan_single_candidate(
                input.context,
                input.event_queue,
                input.candidates.block_at(block_pos),
                |positions| ActionScope::Block(BlockPositions::new(block_pos, positions)),
            );
        }
    }
}
