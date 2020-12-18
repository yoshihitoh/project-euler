use std::error::Error as StdError;

use itertools::Itertools;

use project_euler::sudoku::{Board, BoardLoader, Solver, Square};

fn load_boards() -> Vec<Board<Square>> {
    let s = include_str!("../../assets/p096_sudoku.txt");
    s.lines()
        .filter(|s| !s.starts_with("Grid"))
        .chunks(9) // 9x9
        .into_iter()
        .map(BoardLoader::from_lines)
        // .skip(6)
        // .take(1)
        .collect()
}

fn main() -> Result<(), Box<dyn StdError>> {
    let boards = load_boards();
    let num_boards = boards.len();
    let mut solved = 0;
    let mut failure = 0;
    let enable_debugging = false;
    let debug_steps = false;

    // let enable_debugging = true;
    // let debug_steps = true;

    for (no, board) in boards.into_iter().enumerate().map(|(i, b)| (i + 1, b)) {
        let board_original = board.clone();
        let mut solver = Solver::new(board);

        if enable_debugging {
            solver.board().show();
        }
        let mut updated = 0;
        while solver.update()? {
            updated += 1;

            if enable_debugging && debug_steps {
                println!();
                println!(
                    "################################################################################"
                );
                println!();
                println!("Board (update:{})", updated);
                solver.board().show();
                println!("Candidates (update:{})", updated);
                solver.candidates().show();
            }
        }

        let complete = solver.board().is_complete();
        let status_label = if complete {
            solved += 1;
            "Complete!"
        } else {
            failure += 1;
            "Failure"
        };

        println!("Board#{}: {} (with {} updates)", no, status_label, updated);

        if enable_debugging && !complete {
            println!();
            println!(
                "################################################################################"
            );
            println!("[Board#{}]: update{}", no, updated);
            solver.board().show();
            solver.candidates().show();
        }
    }

    println!("solved: {}/{} (failure:{})", solved, num_boards, failure);
    Ok(())
}
