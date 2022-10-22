use bit_set::BitSet;
use std::iter::Skip;

use super::dlx::Dlx;
use super::node::{IterDown, Node};
use super::Problem;

pub(crate) struct Solver<'a, L> {
    dlx: Dlx<'a>,
    labels: Vec<L>,
}

impl<'a, L> Solver<'a, L> {
    pub(crate) fn new<T>(problem: Problem<'a, L, T>) -> Self {
        Solver {
            dlx: Dlx::new(problem.root),
            labels: problem.labels,
        }
    }

    pub(crate) fn solve(mut self) -> Option<Vec<L>> {
        let sol_indices = AlgorithmX::new(self.dlx).next()?;
        let mut i = 0;
        self.labels.retain(|_| {
            i += 1;
            sol_indices.contains(i - 1)
        });
        Some(self.labels)
    }
}

pub(crate) struct Solutions<'a, L> {
    alg: AlgorithmX<'a>,
    labels: Vec<L>,
}

impl<'a, L: Clone> Iterator for Solutions<'a, L> {
    type Item = Vec<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let sol_indices = self.alg.next()?;
        let solution = self
            .labels
            .iter()
            .enumerate()
            .filter_map(|(i, label)| sol_indices.contains(i).then(|| label.clone()))
            .collect();
        Some(solution)
    }
}

impl<'a, L: Clone> Solver<'a, L> {
    pub(crate) fn solutions(self) -> Solutions<'a, L> {
        Solutions {
            alg: AlgorithmX::new(self.dlx),
            labels: self.labels,
        }
    }
}

struct AlgorithmX<'a> {
    dlx: Dlx<'a>,
    selected_rows: BitSet,
    context: Vec<Context<'a>>,
}

struct Context<'a> {
    selected_row: Option<Node<'a>>,
    candidate_rows: Option<Skip<IterDown<'a>>>,
}

impl<'a> AlgorithmX<'a> {
    fn new(dlx: Dlx<'a>) -> Self {
        let header = dlx.min_size_col();
        let context = vec![Context {
            selected_row: None,
            candidate_rows: header.map(|header| header.iter_down().skip(1)),
        }];

        AlgorithmX {
            dlx,
            selected_rows: BitSet::new(),
            context,
        }
    }

    fn select(&mut self, row: Node<'a>) {
        self.selected_rows.insert(row.ix());
        self.dlx.cover(row);
    }

    fn unselect(&mut self, row: Node<'a>) {
        self.dlx.uncover(row);
        self.selected_rows.remove(row.ix());
    }
}

impl<'a> Iterator for AlgorithmX<'a> {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ctx) = self.context.last_mut() {
            if self.dlx.is_empty() {
                let solution = self.selected_rows.clone();
                if let Some(row) = ctx.selected_row { self.unselect(row) };
                self.context.pop();
                return Some(solution);
            }

            if let Some(row) = ctx.candidate_rows.as_mut().unwrap().next() {
                self.select(row);
                let header = self.dlx.min_size_col();
                self.context.push(Context {
                    selected_row: Some(row),
                    candidate_rows: header.map(|header| header.iter_down().skip(1)),
                });
            } else {
                if let Some(row) = ctx.selected_row { self.unselect(row) };
                self.context.pop();
            }
        }

        None
    }
}
