use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

extern crate dlx;

fn main() {
    let stdin = stdin();
    let stdout = stdout();
    let lines = BufReader::new(stdin.lock()).lines();
    let mut out = BufWriter::new(stdout.lock());

    for line in lines {
        let result = read_board(&line.unwrap()).and_then(dlx::solve);

        if let Some(result) = result {
            write_board(result, &mut out);
        } else {
            writeln!(out, "skip").unwrap();
        }
    }
}

type Cell = (u8, u8);

fn read_board(line: &str) -> Option<Vec<(Cell, Vec<usize>)>> {
    if line.len() != 81 {
        return None;
    }

    let mut constraint = Vec::new();

    for (ix, ch) in line.chars().enumerate() {
        if ch == '.' {
            // empty cell
            (1..=9).for_each(|num| add_constraint(ix as u8, num, &mut constraint));
        } else if let Some(num) = ch.to_digit(10) {
            // filled cell
            add_constraint(ix as u8, num as u8, &mut constraint);
        } else {
            return None;
        }
    }

    Some(constraint)
}

fn add_constraint(ix: u8, num: u8, constraints: &mut Vec<(Cell, Vec<usize>)>) {
    let num_ix = (num - 1) as usize;
    let row_ix = (ix / 9) as usize;
    let col_ix = (ix % 9) as usize;
    let subgrid_ix = (3 * (row_ix / 3) + (col_ix / 3)) as usize;

    let row_col = ix as usize;
    let row_num = 9 * row_ix + num_ix + 81;
    let col_num = 9 * col_ix + num_ix + 162;
    let subgrid_num = 9 * subgrid_ix + num_ix + 243;

    constraints.push(((ix, num), vec![row_col, row_num, col_num, subgrid_num]));
}

fn write_board(mut board: Vec<Cell>, out: &mut impl Write) {
    board.sort_unstable();
    board.into_iter().for_each(|(_, n)| {
        write!(out, "{}", n).unwrap();
    });
    writeln!(out).unwrap();
}
