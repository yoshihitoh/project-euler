use std::fmt;
use std::fmt::Debug;
use std::iter::FromIterator;

use crate::sudoku::digit::Digit;
use crate::sudoku::square::Square;

#[derive(Debug, Clone)]
pub struct Board {
    squares: Vec<Square>,
    box_size: usize,
    num_boxes: usize,
}

impl Board {
    pub fn new(squares: Vec<Square>, box_size: usize, num_boxes: usize) -> Self {
        Board {
            squares,
            box_size,
            num_boxes,
        }
    }

    pub fn with_size(box_size: usize, num_boxes: usize) -> Self {
        let edge_size = box_size * num_boxes;
        let squares = vec![Square::default(); edge_size * edge_size];
        Self::new(squares, box_size, num_boxes)
    }

    pub fn squares(&self) -> impl Iterator<Item = &Square> {
        self.squares.iter()
    }

    pub fn square_at(&self, row: usize, col: usize) -> &Square {
        let index = self.index_of(row, col);
        &self.squares[index]
    }

    pub fn square_at_mut(&mut self, row: usize, col: usize) -> &mut Square {
        let index = self.index_of(row, col);
        &mut self.squares[index]
    }

    pub fn row_squares(&self, row: usize) -> impl Iterator<Item = &Square> {
        let index = self.index_of(row, 0);
        self.squares.iter().skip(index).take(self.width())
    }

    pub fn row_squares_mut(&mut self, row: usize) -> impl Iterator<Item = &mut Square> {
        let index = self.index_of(row, 0);
        let width = self.width();
        self.squares.iter_mut().skip(index).take(width)
    }

    pub fn column_squares(&self, col: usize) -> impl Iterator<Item = &Square> {
        let index = self.index_of(0, col);
        self.squares.iter().skip(index).step_by(self.width())
    }

    pub fn column_squares_mut(&mut self, col: usize) -> impl Iterator<Item = &mut Square> {
        let index = self.index_of(0, col);
        let width = self.width();
        self.squares.iter_mut().skip(index).step_by(width)
    }

    pub fn box_squares(&self, box_row: usize, box_col: usize) -> impl Iterator<Item = &Square> {
        assert!(box_row < self.num_boxes);
        assert!(box_col < self.num_boxes);

        let row = box_row * self.num_boxes;
        let col = box_col * self.num_boxes;

        (0..self.box_size).flat_map(move |row_offset| {
            let index = self.index_of(row + row_offset, col);
            self.squares.iter().skip(index).take(self.box_size)
        })
    }

    pub fn box_squares_mut(
        &mut self,
        box_row: usize,
        box_col: usize,
    ) -> impl Iterator<Item = &mut Square> {
        assert!(box_row < self.num_boxes);
        assert!(box_col < self.num_boxes);

        let row = box_row * self.num_boxes;
        let col = box_col * self.num_boxes;

        let left_top_index = self.index_of(row, col);
        let width = self.width();
        let height = self.height();
        let box_size = self.box_size;

        self.squares
            .iter_mut()
            .enumerate()
            .skip(left_top_index)
            // TODO: takeで終了条件を追加すること、現状はボックス範囲外を抜けてもループしてる
            .filter(move |(index, _)| {
                let r = index / height;
                let c = index % width;

                let row_matches = r >= row && r < row + box_size;
                let col_matches = c >= col && c < col + box_size;

                row_matches && col_matches
            })
            .map(|(_, square)| square)
    }

    // TODO: 名前がわかりづらい、なんとかすること
    //   (box_col=1, row=4) の場合は `*` をとる
    //           [0]    [1]    [2]
    //       +-------+-------+-------+
    //  [0]  | - - - | - - - | - - - |
    //  [1]  | - - - | - - - | - - - |
    //  [2]  | - - - | - - - | - - - |
    //       +-------+-------+-------+
    //  [3]  | - - - | - - - | - - - |
    //  [4]  | - - - | * * * | - - - |
    //  [5]  | - - - | - - - | - - - |
    //       +-------+-------+-------+
    //  [6]  | - - - | - - - | - - - |
    //  [7]  | - - - | - - - | - - - |
    //  [8]  | - - - | - - - | - - - |
    //       +-------+-------+-------+
    pub fn box_columns_with_row(
        &self,
        box_col: usize,
        row: usize,
    ) -> impl Iterator<Item = &Square> {
        assert!(box_col < self.num_boxes);
        assert!(row < self.height());

        let col = box_col * self.num_boxes;
        self.row_squares(row).skip(col).take(self.box_size)
    }

    // TODO: 名前がわかりづらい、なんとかすること
    pub fn box_columns_with_row_mut(
        &mut self,
        box_col: usize,
        row: usize,
    ) -> impl Iterator<Item = &mut Square> {
        assert!(box_col < self.num_boxes);
        assert!(row < self.height());

        let col_offset = box_col * self.num_boxes;
        let box_size = self.box_size;
        self.row_squares_mut(row).skip(col_offset).take(box_size)
    }

    // TODO: 名前がわかりづらい、なんとかすること
    //   (box_row=1, col=3) の場合は `*` をとる
    //         0 1 2   3 4 5   6 7 8
    //       +-------+-------+-------+
    //       | - - - | - - - | - - - |
    //  [0]  | - - - | - - - | - - - |
    //       | - - - | - - - | - - - |
    //       +-------+-------+-------+
    //       | - - - | * - - | - - - |
    //  [1]  | - - - | * - - | - - - |
    //       | - - - | * - - | - - - |
    //       +-------+-------+-------+
    //       | - - - | - - - | - - - |
    //  [2]  | - - - | - - - | - - - |
    //       | - - - | - - - | - - - |
    //       +-------+-------+-------+
    pub fn box_rows_with_column(
        &self,
        box_row: usize,
        col: usize,
    ) -> impl Iterator<Item = &Square> {
        assert!(box_row < self.num_boxes);
        assert!(col < self.width());

        let row_offset = box_row * self.num_boxes;
        self.column_squares(col)
            .skip(row_offset)
            .take(self.box_size)
    }

    pub fn box_rows_with_column_mut(
        &mut self,
        box_row: usize,
        col: usize,
    ) -> impl Iterator<Item = &mut Square> {
        assert!(box_row < self.num_boxes);
        assert!(col < self.width());

        let row_offset = box_row * self.num_boxes;
        let box_size = self.box_size;
        self.column_squares_mut(col).skip(row_offset).take(box_size)
    }

    pub fn clear(&mut self) {
        self.squares = vec![Square::default(); self.squares.len()];
    }

    pub fn box_size(&self) -> usize {
        self.box_size
    }

    pub fn num_boxes(&self) -> usize {
        self.num_boxes
    }

    pub fn width(&self) -> usize {
        self.box_size * self.num_boxes
    }

    pub fn height(&self) -> usize {
        self.box_size * self.num_boxes
    }

    fn index_of(&self, row: usize, col: usize) -> usize {
        row * self.width() + col
    }

    pub fn show(&self) {
        println!("(Board)");
        println!("{}", self);
    }

    pub fn show_candidates(&self) {
        let print_separator = || {
            print!(" +");
            for _ in 0..self.width() {
                for _ in 0..self.box_size() {
                    print!("--");
                }
                print!("-+");
            }
            println!();
        };

        println!("(Candidates)");
        let all_digits = Digit::all_digits();
        for row in 0..self.height() {
            print_separator();

            for offset_base in 0..3 {
                for col in 0..self.width() {
                    let sq = self.square_at(row, col);
                    print!(" |");
                    for d in all_digits.iter().skip(offset_base * 3).take(3) {
                        let s = sq
                            .candidates()
                            .get(d)
                            .map(|d| format!("{:2}", d.get()))
                            .unwrap_or_else(|| String::from("  "));
                        print!("{}", s);
                    }
                }
                println!(" |");
            }
        }
        print_separator();
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let write_separator = |f: &mut fmt::Formatter<'_>| {
            write!(f, " ")?;
            for col in 0..self.width() {
                if col % self.box_size == 0 {
                    write!(f, "+-")?;
                }
                write!(f, "--")?;
            }
            writeln!(f, "+")
        };

        for row in 0..self.height() {
            if row % self.box_size == 0 {
                write_separator(f)?;
            }
            for col in 0..self.width() {
                if col % self.box_size == 0 {
                    write!(f, " |")?;
                }

                write!(f, " {}", self.square_at(row, col))?;
            }
            writeln!(f, " |")?;
        }
        write_separator(f)
    }
}

impl<'a> FromIterator<&'a str> for Board {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let lines = iter.into_iter().collect::<Vec<_>>();
        let height = lines.len();
        let width = height;

        let box_size = (height as f64).sqrt() as usize;
        if height % box_size != 0 {
            panic!(
                "wrong input. width={}, height={}, box_size={}",
                width, height, box_size
            );
        }
        let num_boxes = height / box_size;

        let squares = lines
            .into_iter()
            .into_iter()
            .flat_map(|s: &str| {
                if s.len() != width {
                    panic!(
                        "Wrong line, must contains {} chars, found {} chars on '{}'",
                        width,
                        s.len(),
                        s
                    );
                }
                s.chars().map(Square::from)
            })
            .collect::<Vec<_>>();

        if squares.len() != width * height {
            panic!("Wrong board size. ");
        }

        Board::new(squares, box_size, num_boxes)
    }
}
