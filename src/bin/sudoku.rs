use rustc_hash::FxHashSet;
use std::convert::TryFrom;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

fn main() {
    let stdin = stdin();
    let stdout = stdout();
    let lines = BufReader::new(stdin.lock()).lines();
    let mut out = BufWriter::new(stdout.lock());

    for line in lines {
        if let Some(solution) = solve(&line.unwrap()) {
            writeln!(out, "{}", solution).unwrap();
        } else {
            writeln!(out, "skip").unwrap();
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Cell {
    row: usize,
    col: usize,
    num: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Constraint {
    RowCol(usize, usize),
    RowNum(usize, usize),
    ColNum(usize, usize),
    BoxNum(usize, usize),
}

impl Cell {
    fn constraints(&self) -> FxHashSet<Constraint> {
        let bx = 3 * (self.row / 3) + (self.col / 3);
        dlx::hashset! {
            Constraint::RowCol(self.row, self.col),
            Constraint::RowNum(self.row, self.num),
            Constraint::ColNum(self.col, self.num),
            Constraint::BoxNum(bx, self.num),
        }
    }
}

fn solve(str: &str) -> Option<String> {
    let constraint = parse(str)?;
    let solution = dlx::solve(constraint)?;
    Some(display(&solution))
}

fn parse(str: &str) -> Option<impl Iterator<Item = (Cell, FxHashSet<Constraint>)>> {
    if str.len() != 81 {
        return None;
    }

    let mut cells = Vec::new();
    for (i, ch) in str.chars().enumerate() {
        let row = i / 9;
        let col = i % 9;
        if ch == '.' {
            (1..=9).for_each(|num| cells.push(Cell { row, col, num }));
        } else if let Some(num @ 1..=9) = ch.to_digit(10) {
            cells.push(Cell {
                row,
                col,
                num: num as usize,
            });
        } else {
            return None;
        }
    }

    let constraints = cells.into_iter().map(|cell| {
        let constraints = cell.constraints();
        (cell, constraints)
    });

    Some(constraints)
}

fn display(board: &[Cell]) -> String {
    board
        .iter()
        .map(|cell| char::from_digit(u32::try_from(cell.num).unwrap(), 10).unwrap())
        .collect()
}
