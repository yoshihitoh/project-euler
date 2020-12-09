use crate::sudoku::action::{ActionScope, RetainAction};
use crate::sudoku::candidate::Candidate;
use crate::sudoku::event::{Event, EventQueue};
use crate::sudoku::filter::{
    FilterCandidates, FilterContext, FilterInput, NamedFilter, ScanCandidates,
};
use crate::sudoku::positions::{BlockPositions, ColumnPositions, Positions, RowPositions};
use crate::sudoku::set::DigitSet;
use itertools::Itertools;
use std::iter::FromIterator;

struct HiddenCandidates;

impl HiddenCandidates {
    fn search_hidden_digits<'a, I, F>(
        &self,
        context: &mut FilterContext,
        event_queue: &mut EventQueue,
        num_pairs: usize,
        num_candidates: usize,
        candidates: I,
        scope_fn: F,
    ) -> Option<(DigitSet, Positions)>
    where
        I: Iterator<Item = &'a Candidate>,
        F: Fn(Positions) -> ActionScope,
    {
        // 候補を覚えておく
        // - 数字ごとの配置を調べる
        context.collect_digit_positions_matches(candidates, |_, ps| ps.num_set() >= 2);

        // - 2つ以上のセルで使ってる数値を列挙する
        let hidden_pairs = context
            .digit_positions
            .iter()
            .map(|(d, _)| d)
            .combinations(num_pairs)
            .map(|digits| {
                // ペアを探す
                let positions = digits.iter().fold(
                    Positions::default().invert(num_candidates),
                    |positions, d| {
                        context
                            .digit_positions
                            .get(*d)
                            .map(|ps| positions.and(ps))
                            .unwrap_or(positions)
                    },
                );
                (digits, positions)
            })
            .filter(|(_, positions)| {
                // 組の配置場所の数が一致するか検証する
                positions.num_set() == num_pairs
            })
            .filter(|(digits, positions)| {
                // 他の候補に含まれていないか検証する
                digits.iter().all(|d| {
                    let digit_positions = context.digit_positions.get(*d).copied();
                    // dbg!((&digits, d, digit_positions, positions));
                    digit_positions
                        .map(|ps| *positions == ps)
                        .expect("never happen")
                })
            });

        for (digits, positions) in hidden_pairs {
            let digits = DigitSet::from_iter(digits.iter().map(|d| **d));
            let scope = scope_fn(positions);
            let action = RetainAction::new(digits, scope);
            let event = Event::from(action);
            event_queue.push_back(event);
        }

        // - 数値の組を作って確認する
        //     - 一致する組を列挙する
        //     - 一致した数が要素数と正しいか検証する
        //     - 一致する場合は確定、その他の候補から組を構成する数字を削除する

        None
    }
}

pub trait HiddenCombinations: FilterCandidates {
    fn num_combinations(&self) -> usize;
}

impl<T: HiddenCombinations> ScanCandidates for T {
    fn scan_rows(&self, input: &mut FilterInput) {
        for row in input.board.each_rows() {
            HiddenCandidates.search_hidden_digits(
                input.context,
                input.event_queue,
                self.num_combinations(),
                input.candidates.height(),
                input.candidates.row_items(row),
                |positions| ActionScope::Row(RowPositions::new(row, positions)),
            );
        }
    }

    fn scan_columns(&self, input: &mut FilterInput) {
        for col in input.board.each_columns() {
            HiddenCandidates.search_hidden_digits(
                input.context,
                input.event_queue,
                self.num_combinations(),
                input.candidates.width(),
                input.candidates.column_items(col),
                |positions| ActionScope::Column(ColumnPositions::new(col, positions)),
            );
        }
    }

    fn scan_blocks(&self, input: &mut FilterInput) {
        for block_pos in input.board.block_positions() {
            HiddenCandidates.search_hidden_digits(
                input.context,
                input.event_queue,
                self.num_combinations(),
                input.candidates.block_item_indexes().len(),
                input.candidates.block_at(block_pos),
                |positions| ActionScope::Block(BlockPositions::new(block_pos, positions)),
            );
        }
    }
}

pub struct HiddenPair;

impl NamedFilter for HiddenPair {
    fn name(&self) -> &'static str {
        "HiddenPair"
    }
}

impl HiddenCombinations for HiddenPair {
    fn num_combinations(&self) -> usize {
        2
    }
}

pub struct HiddenTriple;

impl NamedFilter for HiddenTriple {
    fn name(&self) -> &'static str {
        "HiddenTriple"
    }
}

impl HiddenCombinations for HiddenTriple {
    fn num_combinations(&self) -> usize {
        3
    }
}

pub struct HiddenQuad;

impl NamedFilter for HiddenQuad {
    fn name(&self) -> &'static str {
        "HiddenQuad"
    }
}

impl HiddenCombinations for HiddenQuad {
    fn num_combinations(&self) -> usize {
        4
    }
}
