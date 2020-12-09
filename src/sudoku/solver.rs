use std::collections::BTreeSet;
use std::iter::FromIterator;

use crate::sudoku::board::Board;
use crate::sudoku::digit::Digit;

pub struct Solver {
    board: Board,
}

impl Solver {
    pub fn new(board: Board) -> Solver {
        Solver { board }
    }

    pub fn is_solved(&self) -> bool {
        self.board.squares().all(|sq| sq.is_fixed())
    }

    pub fn update(&mut self) -> bool {
        let mut updated = false;
        updated = self.update_rows() || updated;
        updated = self.update_columns() || updated;
        updated = self.update_boxes() || updated;

        updated = self.update_box_rows() || updated;
        updated = self.update_box_columns() || updated;
        updated
    }

    fn update_rows(&mut self) -> bool {
        let height = self.board.height();
        (0..height).fold(false, |updated, row| self.update_row(row) || updated)
    }

    fn update_row(&mut self, row: usize) -> bool {
        let digits_in_use =
            BTreeSet::from_iter(self.board.row_squares(row).filter_map(|sq| sq.digit()));

        let mut updated = self
            .board
            .row_squares_mut(row)
            .filter(|sq| !sq.is_fixed())
            .fold(false, |updated, sq| {
                sq.remove_candidates_iter(digits_in_use.iter().copied()) || updated
            });

        // 未確定の数字について、ボックス内で候補が1個だけになったら確定する
        let mut indexes = Vec::new();
        for digit_in_progress in &Digit::all_digits() - &digits_in_use {
            indexes.clear();
            indexes.extend(
                self.board
                    .row_squares(row)
                    .enumerate()
                    .filter_map(|(i, sq)| sq.candidates().get(&digit_in_progress).map(|_| i)),
            );

            if indexes.len() == 1 {
                self.board
                    .row_squares_mut(row)
                    .skip(indexes[0])
                    .next()
                    .unwrap()
                    .fix_digit(digit_in_progress);
                updated = true;
            }
        }

        updated
    }

    fn update_box_rows(&mut self) -> bool {
        let num_boxes = self.board.num_boxes();
        let box_size = self.board.box_size();
        (0..num_boxes).fold(false, move |updated, box_row| {
            self.update_box_row(box_row, box_size, num_boxes) || updated
        })
    }

    fn update_box_row(&mut self, box_row: usize, box_size: usize, num_boxes: usize) -> bool {
        // 3x3のみ対応
        assert!(self.board.box_size() == 3);
        assert!(self.board.num_boxes() == 3);

        let mut updated = false;

        // ボードがこういう状態の場合、一番下のボックス列の7に注目する
        // 右下の2行目は埋まっているため、右下1行目の*のどちらかに入る
        // そのため、2行目の7は左下のボックスの#のどちらかになる
        //      +-------+-------+-------+
        // [0]  | 1 - - | 9 2 - | - - - |
        // [1]  | 5 2 4 | - 1 7 | - - 9 |
        // [2]  | - - - | - - - | 2 7 1 |
        //      +-------+-------+-------+
        // [3]  | - 5 - | - - 8 | 1 - 2 |
        // [4]  | - - - | 1 - 2 | - - - |
        // [5]  | 4 1 2 | 7 - - | - 9 - |
        //      +-------+-------+-------+
        // [6]  | - 6 - | - - 9 | * 1 * |
        // [7]  | # # 1 | - 3 6 | 9 4 5 |
        // [8]  | - 4 - | - 7 1 | - 2 6 |
        //      +-------+-------+-------+
        //
        // 以下が関係するボックスの候補、左下ボックスの1行目の候補から 7 を消せる
        //  +-------+-------+-------+-------+-------+-------+-------+-------+-------+
        //  |   2 3 |       |     3 |   2   |       |       |     3 |       |     3 |
        //  |       |       |   5   | 4 5   | 4 5   |       |       |       |       |
        //  | 7 8   |       | 7 8   |   8   |   8   |       | 7 8   |       | 7 8   |
        //  +-------+-------+-------+-------+-------+-------+-------+-------+-------+
        //  |   2   |       |       |   2   |       |       |       |       |       |
        //  |       |       |       |       |       |       |       |       |       |
        //  | 7 8   | 7 8   |       |   8   |       |       |       |       |       |
        //  +-------+-------+-------+-------+-------+-------+-------+-------+-------+
        //  |     3 |       |     3 |       |       |       |     3 |       |       |
        //  |       |       |   5   |   5   |       |       |       |       |       |
        //  |   8 9 |       |   8 9 |   8   |       |       |   8   |       |       |
        //  +-------+-------+-------+-------+-------+-------+-------+-------+-------+
        //
        // if ボックス内の行or列が埋まっている場合:
        //     for d in 他の行・他のボックスで使っている数字を列挙する:
        //     候補 = 候補 - 埋まってる行or列の数字
        //

        let mut digits_in_use = BTreeSet::new();
        let mut indexes = Vec::new();
        let row_top = box_row * box_size;
        let row_bottom = row_top + box_size;
        for row in row_top..row_bottom {
            for box_col in 0..num_boxes {
                digits_in_use.clear();
                digits_in_use.extend(
                    self.board
                        .box_columns_with_row(box_col, row)
                        .filter_map(|sq| sq.digit()),
                );
                if digits_in_use.len() != box_size {
                    continue;
                }

                // 全カラム確定してる場合、該当カラムの候補からその他ボックスで使ってる数字を除外する
                // ボックス列中で1個しか使ってない数字が対象か？
                // → 一旦全部試す方向で
                for ref_row in (row_top..row_bottom).filter(|&r| r != row) {
                    for ref_box_col in (0..num_boxes).filter(|&bc| bc != box_col) {
                        indexes.clear();
                        indexes.extend(
                            self.board
                                .box_columns_with_row(ref_box_col, ref_row)
                                .enumerate()
                                .filter_map(|(i, sq)| {
                                    sq.digit().and_then(|d| {
                                        digits_in_use.get(&d).map(|_| None).unwrap_or(Some(i))
                                    })
                                }),
                        );

                        for &i in indexes.iter() {
                            let d = self
                                .board
                                .box_columns_with_row(ref_box_col, ref_row)
                                .skip(i)
                                .next()
                                .and_then(|sq| sq.digit())
                                .unwrap();

                            // 残りのボックスの候補から除外する
                            let dest_box_col = box_size - (box_col + ref_box_col);
                            // dbg!((row, box_col, ref_row, ref_box_col, &indexes, dest_box_col));
                            updated = self
                                .board
                                .box_columns_with_row_mut(dest_box_col, row)
                                .fold(updated, |updated, sq| sq.remove_candidate(d) || updated);
                        }
                    }
                }
            }
        }

        updated
    }

    fn update_columns(&mut self) -> bool {
        let width = self.board.width();
        (0..width).fold(false, |updated, col| self.update_column(col) || updated)
    }

    fn update_column(&mut self, col: usize) -> bool {
        let digits_in_use =
            BTreeSet::from_iter(self.board.column_squares(col).filter_map(|sq| sq.digit()));

        let mut updated = self
            .board
            .column_squares_mut(col)
            .filter(|sq| !sq.is_fixed())
            .fold(false, |updated, sq| {
                sq.remove_candidates_iter(digits_in_use.iter().copied()) || updated
            });

        // 未確定の数字について、ボックス内で候補が1個だけになったら確定する
        let mut indexes = Vec::new();
        for digit_in_progress in &Digit::all_digits() - &digits_in_use {
            indexes.clear();
            indexes.extend(
                self.board
                    .column_squares(col)
                    .enumerate()
                    .filter_map(|(i, sq)| sq.candidates().get(&digit_in_progress).map(|_| i)),
            );

            if indexes.len() == 1 {
                self.board
                    .column_squares_mut(col)
                    .skip(indexes[0])
                    .next()
                    .unwrap()
                    .fix_digit(digit_in_progress);
                updated = true;
            }
        }

        updated
    }

    fn update_box_columns(&mut self) -> bool {
        let num_boxes = self.board.num_boxes();
        let box_size = self.board.box_size();
        (0..num_boxes).fold(false, move |updated, box_col| {
            self.update_box_column(box_col, box_size, num_boxes) || updated
        })
    }

    fn update_box_column(&mut self, box_row: usize, box_size: usize, num_boxes: usize) -> bool {
        // 3x3のみ対応
        assert!(self.board.box_size() == 3);
        assert!(self.board.num_boxes() == 3);

        let mut updated = false;
        let mut digits_in_use = BTreeSet::new();
        let mut indexes = Vec::new();
        let col_left = box_row * box_size;
        let col_right = col_left + box_size;
        for col in col_left..col_right {
            for box_row in 0..num_boxes {
                digits_in_use.clear();
                digits_in_use.extend(
                    self.board
                        .box_rows_with_column(box_row, col)
                        .filter_map(|sq| sq.digit()),
                );
                if digits_in_use.len() != box_size {
                    continue;
                }

                // 全カラム確定してる場合、該当カラムの候補からその他ボックスで使ってる数字を除外する
                // ボックス列中で1個しか使ってない数字が対象か？
                // → 一旦全部試す方向で
                for ref_col in (col_left..col_right).filter(|&c| c != col) {
                    for ref_box_row in (0..num_boxes).filter(|&br| br != box_row) {
                        indexes.clear();
                        indexes.extend(
                            self.board
                                .box_rows_with_column(ref_box_row, ref_col)
                                .enumerate()
                                .filter_map(|(i, sq)| {
                                    sq.digit().and_then(|d| {
                                        digits_in_use.get(&d).map(|_| None).unwrap_or(Some(i))
                                    })
                                }),
                        );

                        for &i in indexes.iter() {
                            let d = self
                                .board
                                .box_rows_with_column(ref_box_row, ref_col)
                                .skip(i)
                                .next()
                                .and_then(|sq| sq.digit())
                                .unwrap();

                            // 残りのボックスの候補から除外する
                            let dest_box_row = box_size - (box_row + ref_box_row);
                            // dbg!((row, box_row, ref_col, ref_box_row, &indexes, dest_box_row));
                            updated = self
                                .board
                                .box_rows_with_column_mut(dest_box_row, col)
                                .fold(updated, |updated, sq| sq.remove_candidate(d) || updated);
                        }
                    }
                }
            }
        }

        updated
    }

    fn update_boxes(&mut self) -> bool {
        let box_size = self.board.num_boxes();
        (0..box_size)
            .flat_map(|box_row| (0..box_size).map(move |box_col| (box_row, box_col)))
            .fold(false, |updated, (box_row, box_col)| {
                self.update_box(box_row, box_col) || updated
            })
    }

    fn update_box(&mut self, box_row: usize, box_col: usize) -> bool {
        // 確定してる数字を候補から消す
        let digits_in_use = BTreeSet::from_iter(
            self.board
                .box_squares(box_row, box_col)
                .filter_map(|sq| sq.digit()),
        );

        let mut updated = self
            .board
            .box_squares_mut(box_row, box_col)
            .filter(|sq| !sq.is_fixed())
            .fold(false, |updated, sq| {
                sq.remove_candidates_iter(digits_in_use.iter().copied()) || updated
            });

        // 未確定の数字について、ボックス内で候補が1個だけになったら確定する
        let mut indexes = Vec::new();
        for digit_in_progress in &Digit::all_digits() - &digits_in_use {
            indexes.clear();
            indexes.extend(
                self.board
                    .box_squares(box_row, box_col)
                    .enumerate()
                    .filter_map(|(i, sq)| sq.candidates().get(&digit_in_progress).map(|_| i)),
            );

            if indexes.len() == 1 {
                self.board
                    .box_squares_mut(box_row, box_col)
                    .skip(indexes[0])
                    .next()
                    .unwrap()
                    .fix_digit(digit_in_progress);
                updated = true;
            }
        }

        updated
    }

    pub fn show_board(&self) {
        self.board.show();
    }

    pub fn show_candidates(&self) {
        self.board.show_candidates();
    }
}
