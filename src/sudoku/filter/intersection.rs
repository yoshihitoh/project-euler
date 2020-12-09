use std::iter::FromIterator;
use std::ops::Range;

use crate::sudoku::action::{ActionScope, RemoveAction};
use crate::sudoku::board::BlockPosition;
use crate::sudoku::candidate::Candidate;
use crate::sudoku::event::{Event, EventQueue};
use crate::sudoku::filter::{FilterContext, FilterInput, NamedFilter, ScanCandidates};
use crate::sudoku::positions::{BlockPositions, ColumnPositions, Positions, RowPositions};

struct LockedCandidate {
    block_size: usize,
    num_blocks: usize,
}

impl LockedCandidate {
    fn positions_exclude_block(&self, block_offset: usize) -> Positions {
        let item_start = block_offset * self.block_size;
        let excludes = item_start..(item_start + self.block_size);
        let iter = (0..(self.block_size * self.num_blocks)).filter(|i| !excludes.contains(i));
        Positions::from_iter(iter)
    }

    fn scan_pointing<'a, FI, I, FS>(
        &self,
        context: &mut FilterContext,
        event_queue: &mut EventQueue,
        block_offsets: Range<usize>,
        block_candidates: impl Iterator<Item = &'a Candidate>,
        iter_fn: FI,
        scope_fn: FS,
    ) where
        FI: Fn(usize) -> I,
        I: Iterator<Item = &'a Candidate>,
        FS: Fn(usize) -> ActionScope,
    {
        context.collect_digit_positions_matches(block_candidates, |_, positions| {
            positions.num_set() >= 2 || positions.num_set() <= self.block_size
        });

        for (digit, positions) in context.digit_positions.iter() {
            for offset in block_offsets.clone() {
                let num_digits = iter_fn(offset).filter(|c| c.contains(*digit)).count();
                if num_digits == positions.num_set() {
                    // ブロックのどちらかに入るので、その他のブロックの列・行から除外する
                    let scope = scope_fn(offset);
                    let action = RemoveAction::new(*digit, scope);
                    let event = Event::from(action);
                    event_queue.push_back(event);
                }
            }
        }
    }

    fn scan_claiming<'a, F>(
        &self,
        context: &mut FilterContext,
        event_queue: &mut EventQueue,
        block_offsets: Range<usize>,
        candidates: impl Iterator<Item = &'a Candidate>,
        scope_fn: F,
    ) where
        F: Fn(usize) -> ActionScope,
    {
        context.collect_digit_positions_matches(candidates, |_, positions| {
            positions.num_set() >= 2 && positions.num_set() <= self.block_size
        });

        for (digit, positions) in context.digit_positions.iter() {
            // 同一のブロック内に収まっている場合は、その他のブロックの候補から削除する
            for offset in block_offsets.clone() {
                let item_start = self.block_size * offset;
                let block_positions = Positions::with_offset(item_start, self.block_size);
                if positions.belongs_to(&block_positions) {
                    // すべての候補が同一ブロックに存在する → 同ブロックのその他の行・列から削除する
                    let scope = scope_fn(offset);
                    let action = RemoveAction::new(*digit, scope);
                    let event = Event::from(action);
                    event_queue.push_back(event);
                }
            }
        }
    }
}

pub struct LockedCandidatePointing;

impl NamedFilter for LockedCandidatePointing {
    fn name(&self) -> &'static str {
        "LockedCandidate(Pointing)"
    }
}

impl ScanCandidates for LockedCandidatePointing {
    fn scan_rows(&self, input: &mut FilterInput) {
        let block_size = input.board.block_size();
        let locked_candidate = LockedCandidate {
            block_size,
            num_blocks: input.board.num_blocks(),
        };

        for block_pos in input.board.block_positions() {
            let candidates = input.candidates;
            locked_candidate.scan_pointing(
                input.context,
                input.event_queue,
                input.board.each_block_rows(),
                input.candidates.block_at(block_pos),
                |block_row| candidates.block_at(block_pos).row_items(block_row),
                |block_row| {
                    let row = block_pos.row * block_size + block_row;
                    let positions = locked_candidate.positions_exclude_block(block_pos.col);
                    ActionScope::Row(RowPositions::new(row, positions))
                },
            );
        }
    }

    fn scan_columns(&self, input: &mut FilterInput) {
        let block_size = input.board.block_size();
        let locked_candidate = LockedCandidate {
            block_size,
            num_blocks: input.board.num_blocks(),
        };

        for block_pos in input.board.block_positions() {
            let candidates = input.candidates;
            locked_candidate.scan_pointing(
                input.context,
                input.event_queue,
                input.board.each_block_columns(),
                input.candidates.block_at(block_pos),
                |block_col| candidates.block_at(block_pos).column_items(block_col),
                |block_col| {
                    let col = block_pos.col * block_size + block_col;
                    let positions = locked_candidate.positions_exclude_block(block_pos.row);
                    ActionScope::Column(ColumnPositions::new(col, positions))
                },
            );
        }
    }

    fn scan_blocks(&self, _input: &mut FilterInput) {}
}

pub struct LockedCandidateClaiming;

impl NamedFilter for LockedCandidateClaiming {
    fn name(&self) -> &'static str {
        "LockedCandidate(Claiming)"
    }
}

impl ScanCandidates for LockedCandidateClaiming {
    fn scan_rows(&self, input: &mut FilterInput) {
        let block_size = input.board.block_size();
        let num_blocks = input.board.num_blocks();
        let locked_candidate = LockedCandidate {
            block_size,
            num_blocks,
        };

        for row in input.board.each_rows() {
            locked_candidate.scan_claiming(
                input.context,
                input.event_queue,
                input.board.each_block_columns(),
                input.candidates.row_items(row),
                |block_col| {
                    let block_row = row / block_size;
                    let block_at = BlockPosition {
                        row: block_row,
                        col: block_col,
                    };
                    let positions =
                        Positions::with_offset((row % block_size) * block_size, block_size)
                            .invert(block_size * num_blocks);
                    ActionScope::Block(BlockPositions::new(block_at, positions))
                },
            );
        }
    }

    fn scan_columns(&self, input: &mut FilterInput) {
        let block_size = input.board.block_size();
        let num_blocks = input.board.num_blocks();
        let locked_candidate = LockedCandidate {
            block_size,
            num_blocks,
        };

        for col in input.board.each_columns() {
            let block_item_indexes = input.board.block_item_indexes();
            locked_candidate.scan_claiming(
                input.context,
                input.event_queue,
                input.board.each_block_rows(),
                input.candidates.column_items(col),
                |block_row| {
                    let block_col = col / block_size;
                    let block_at = BlockPosition {
                        row: block_row,
                        col: block_col,
                    };
                    let iter = block_item_indexes
                        .clone()
                        .skip(col % block_size)
                        .step_by(block_size);
                    let positions = Positions::from_iter(iter).invert(block_size * num_blocks);
                    ActionScope::Block(BlockPositions::new(block_at, positions))
                },
            );
        }
    }

    fn scan_blocks(&self, _input: &mut FilterInput) {}
}
