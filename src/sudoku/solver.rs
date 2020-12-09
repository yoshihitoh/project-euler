use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;

use thiserror::Error;

use crate::sudoku::board::{Board, BoardError};
use crate::sudoku::candidate::Candidate;
use crate::sudoku::event::EventQueue;
use crate::sudoku::filter::{
    FilterCandidates, FilterContext, FilterInput, HiddenPair, HiddenQuad, HiddenTriple,
    LockedCandidateClaiming, LockedCandidatePointing, NakedSingle, SingleCandidate,
};
use crate::sudoku::Square;

pub fn add_filter<F: FilterCandidates + 'static>(
    filters: &mut Vec<Box<dyn FilterCandidates>>,
    f: F,
) {
    filters.push(Box::new(f));
}

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("board error")]
    Board {
        #[from]
        source: BoardError,
    },
}

#[derive(Clone)]
struct State {
    board: RefCell<Board<Square>>,
    candidates: RefCell<Board<Candidate>>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Statistics {
    pub get_stuck: usize,
    pub back_tracked: usize,
}

pub struct Solver {
    context: RefCell<FilterContext>,
    event_queue: RefCell<EventQueue>,
    current_state: State,
    possible_states: RefCell<VecDeque<State>>,
    stuck_state: Option<State>,
    filters: Vec<Box<dyn FilterCandidates>>,
    statistics: Statistics,
}

impl Solver {
    fn filters() -> Vec<Box<dyn FilterCandidates>> {
        let mut filters = Vec::new();
        add_filter(&mut filters, NakedSingle);
        add_filter(&mut filters, SingleCandidate);
        add_filter(&mut filters, LockedCandidatePointing);
        add_filter(&mut filters, LockedCandidateClaiming);
        add_filter(&mut filters, HiddenPair);
        add_filter(&mut filters, HiddenTriple);
        add_filter(&mut filters, HiddenQuad);
        filters
    }

    pub fn new(board: Board<Square>) -> Self {
        let items = board.items().map(Candidate::new).collect();
        let candidates = Board::new(items, board.block_size(), board.num_blocks());
        let filters = Self::filters();

        Solver {
            context: RefCell::new(FilterContext::default()),
            event_queue: RefCell::new(EventQueue::default()),
            current_state: State {
                board: RefCell::new(board),
                candidates: RefCell::new(candidates),
            },
            possible_states: RefCell::new(VecDeque::new()),
            stuck_state: None,
            filters,
            statistics: Statistics::default(),
        }
    }

    pub fn board(&self) -> Ref<Board<Square>> {
        self.current_state.board.borrow()
    }

    pub fn board_mut(&self) -> RefMut<Board<Square>> {
        self.current_state.board.borrow_mut()
    }

    pub fn candidates(&self) -> Ref<Board<Candidate>> {
        self.current_state.candidates.borrow()
    }

    fn candidates_mut(&self) -> RefMut<Board<Candidate>> {
        self.current_state.candidates.borrow_mut()
    }

    pub fn statistics(&self) -> Statistics {
        self.statistics
    }

    pub fn update(&mut self) -> Result<bool, SolverError> {
        let mut evaluated = false;
        for filter in self.filters.iter() {
            filter.filter_candidates(FilterInput::new(
                &mut self.context.borrow_mut(),
                &mut self.event_queue.borrow_mut(),
                &self.board(),
                &self.candidates(),
            ));

            evaluated = self.evaluate_events();
            if evaluated {
                break;
            }
        }

        if evaluated {
            if let Err(err) = self.update_board() {
                // 更新中にエラーが発生した場合は他のパターンがあれば試す、なければ断念
                if self.possible_states.borrow().is_empty() {
                    Err(err)?;
                }
            }
        }

        if !evaluated && !self.board().is_complete() {
            // フィルタで解けなくなった、Grid#7 でだけ発生するはず
            // - 手詰まりの状態を保存する
            // - 2択・3択と候補を増やしながらリトライする
            if self.possible_states.borrow().is_empty() {
                self.statistics.get_stuck += 1;
                self.update_possibilities();
            } else {
                self.statistics.back_tracked += 1;
                self.current_state = self.possible_states.borrow_mut().pop_front().unwrap();
            }

            evaluated = true;
        }

        Ok(evaluated)
    }

    fn evaluate_events(&self) -> bool {
        let mut evaluated = false;
        // println!("########################################################################################## START EVALUATING");
        while let Some(mut event) = self.event_queue.borrow_mut().pop_front() {
            // dbg!(event);
            // self.board().show();
            // self.candidates().show();
            evaluated |= event.evaluate(&mut self.candidates_mut());
        }
        // println!("########################################################################################## FINISH EVALUATING");
        // self.board().show();
        // self.candidates().show();
        // println!();
        evaluated
    }

    fn update_board(&self) -> Result<(), BoardError> {
        let positions = self.candidates().item_positions();
        for pos in positions {
            if let Some(digit) = self.candidates_mut().take_fixed_digit_at(pos) {
                self.board_mut().fix_digit_at(pos, digit)?;
            }
        }

        Ok(())
    }

    fn update_possibilities(&mut self) {
        // 手詰まりの状態を保存する
        assert!(self.stuck_state.is_none());
        self.stuck_state = Some(self.current_state.clone());

        // 可能性のある状態を登録する
        let candidates = self.candidates();
        for pos in candidates.item_positions().filter(|pos| {
            self.candidates().item_at(*pos).digits().len() == self.statistics.get_stuck + 1
        }) {
            let c = candidates.item_at(pos);
            let possible_states = c.digits().iter().map(|d| {
                let mut next_candidates = self.candidates().clone();
                next_candidates.item_at_mut(pos).remove(d);

                State {
                    board: RefCell::new(self.board().clone()),
                    candidates: RefCell::new(next_candidates),
                }
            });
            self.possible_states.borrow_mut().extend(possible_states);
        }
    }
}
