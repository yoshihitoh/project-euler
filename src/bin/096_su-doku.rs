use std::error::Error as StdError;
use std::iter::FromIterator;

use itertools::Itertools;
use project_euler::sudoku::{Board, Solver};

fn main() -> Result<(), Box<dyn StdError>> {
    let s = include_str!("../../assets/p096_sudoku.txt");
    let boards: Vec<Board> = s
        .lines()
        .filter(|s| !s.starts_with("Grid"))
        .chunks(9) // 9x9
        .into_iter()
        .map(Board::from_iter)
        .collect();

    let mut solved = 0;
    let mut failure = 0;
    let num_boards = boards.len();
    for (i, b) in boards.into_iter().enumerate() {
        let original = b.clone();
        let mut solver = Solver::new(b);
        let mut updated = 0;
        while solver.update() {
            updated += 1;
            // println!();
            // println!(
            //     "################################################################################"
            // );
            // println!("## updated:{}", updated);
        }
        if solver.is_solved() {
            solved += 1;
        } else {
            failure += 1;
        }

        if !solver.is_solved() {
            println!(
                "================================================================================"
            );

            original.show();
            println!();
            solver.show_board();
            solver.show_candidates();
        }
    }

    println!("solved: {}/{} (failure:{})", solved, num_boards, failure);
    Ok(())
}
