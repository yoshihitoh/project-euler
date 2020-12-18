use thiserror::Error;

use crate::sudoku::board::{Board, BoardError};
use crate::sudoku::candidate::Candidate;
use crate::sudoku::event::EventQueue;
use crate::sudoku::filter::{
    FilterCandidates, FilterContext, FilterInput, LockedCandidate, NakedSingle, NamedFilter,
    SingleCandidate,
};
use crate::sudoku::Square;
use std::cell::{Ref, RefCell, RefMut};

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

pub struct Solver {
    context: RefCell<FilterContext>,
    event_queue: RefCell<EventQueue>,
    board: RefCell<Board<Square>>,
    candidates: RefCell<Board<Candidate>>,
    filters: Vec<Box<dyn FilterCandidates>>,
}

impl Solver {
    pub fn new(board: Board<Square>) -> Self {
        let items = board.items().map(Candidate::new).collect();
        let candidates = Board::new(items, board.box_size(), board.num_boxes());
        let mut filters = Vec::new();
        add_filter(&mut filters, NakedSingle);
        add_filter(&mut filters, SingleCandidate);
        add_filter(&mut filters, LockedCandidate);

        Solver {
            context: RefCell::new(FilterContext::default()),
            event_queue: RefCell::new(EventQueue::default()),
            board: RefCell::new(board),
            candidates: RefCell::new(candidates),
            filters,
        }
    }

    pub fn board(&self) -> Ref<Board<Square>> {
        self.board.borrow()
    }

    pub fn candidates(&self) -> Ref<Board<Candidate>> {
        self.candidates.borrow()
    }

    fn candidates_mut(&self) -> RefMut<Board<Candidate>> {
        self.candidates.borrow_mut()
    }

    pub fn update(&mut self) -> Result<bool, SolverError> {
        let mut evaluated = false;
        for filter in self.filters.iter() {
            filter.filter_candidates(FilterInput::new(
                &mut self.context.borrow_mut(),
                &mut self.event_queue.borrow_mut(),
                &self.board.borrow(),
                &self.candidates.borrow(),
            ));

            evaluated = self.evaluate_events();
            if evaluated {
                break;
            }
        }

        if evaluated {
            self.update_board()?;
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
                self.board.borrow_mut().fix_digit_at(pos, digit)?;
            }
        }

        Ok(())
    }
}
