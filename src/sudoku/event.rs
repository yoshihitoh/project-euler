use std::collections::VecDeque;

use crate::sudoku::action::{RemoveAction, RetainAction};
use crate::sudoku::candidate::Candidate;
use crate::sudoku::Board;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    RetainAction(RetainAction),
    RemoveAction(RemoveAction),
}

impl From<RetainAction> for Event {
    fn from(action: RetainAction) -> Self {
        Event::RetainAction(action)
    }
}

impl From<RemoveAction> for Event {
    fn from(action: RemoveAction) -> Self {
        Event::RemoveAction(action)
    }
}

impl Event {
    pub fn evaluate(&mut self, candidates: &mut Board<Candidate>) -> bool {
        match *self {
            Event::RetainAction(action) => action.retain(candidates),
            Event::RemoveAction(action) => action.remove(candidates),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventQueue(VecDeque<Event>);

impl EventQueue {
    pub fn push_back(&mut self, event: Event) {
        self.0.push_back(event);
    }

    pub fn pop_front(&mut self) -> Option<Event> {
        self.0.pop_front()
    }
}
