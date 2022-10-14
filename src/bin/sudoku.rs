use rustc_hash::FxHashSet;
use std::collections::HashSet;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

fn main() {
    let lines = BufReader::new(stdin().lock()).lines();
    let mut out = BufWriter::new(stdout().lock());

    for line in lines {
        if let Some(solution) = solve(&line.unwrap()) {
            writeln!(out, "{}", solution).unwrap();
        } else {
            writeln!(out, "no solution").unwrap();
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
        HashSet::from_iter(
            [
                Constraint::RowCol(self.row, self.col),
                Constraint::RowNum(self.row, self.num),
                Constraint::ColNum(self.col, self.num),
                Constraint::BoxNum(bx, self.num),
            ]
            .into_iter(),
        )
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
    for (i, ch) in str.char_indices() {
        let row = i / 9;
        let col = i % 9;
        match ch {
            '.' => (1..=9).for_each(|num| cells.push(Cell { row, col, num })),
            num @ '1'..='9' => cells.push(Cell {
                row,
                col,
                num: num.to_digit(10).unwrap().try_into().unwrap(),
            }),
            _ => return None,
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
        .map(|cell| char::from_digit(cell.num.try_into().unwrap(), 10).unwrap())
        .collect()
}
