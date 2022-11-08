use bitvec::vec::BitVec;
use std::cell::Cell;

use crate::dlx::Dlx;
use crate::node::{Column, Node};
use crate::Problem;

pub(crate) struct Solver<'a, L> {
    alg: AlgorithmX<'a>,
    labels: Vec<L>,
}

impl<'a, L> Solver<'a, L> {
    pub(crate) fn new<T>(problem: Problem<'a, L, T>) -> Self {
        Solver {
            alg: AlgorithmX::new(Dlx::new(problem.root), problem.labels.len()),
            labels: problem.labels,
        }
    }

    pub(crate) fn solve(mut self) -> Option<Vec<L>> {
        let select_flags = self.alg.next()?;
        let mut solution: Vec<_> = select_flags
            .iter_ones()
            .rev()
            .map(|i| self.labels.drain(i..).next().unwrap())
            .collect();
        solution.reverse();
        Some(solution)
    }

    pub(crate) fn solutions(self) -> Solutions<'a, L>
    where
        L: Clone,
    {
        Solutions {
            alg: self.alg,
            labels: self.labels,
        }
    }
}

pub(crate) struct Solutions<'a, L> {
    alg: AlgorithmX<'a>,
    labels: Vec<L>,
}

impl<'a, L> Iterator for Solutions<'a, L>
where
    L: Clone,
{
    type Item = Vec<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let select_flags = self.alg.next()?;
        let solution = select_flags
            .iter_ones()
            .map(|i| self.labels[i].clone())
            .collect();
        Some(solution)
    }
}

struct AlgorithmX<'a> {
    dlx: Dlx<'a>,
    init: bool,
    select_flags: BitVec<Cell<usize>>,
    stack: Vec<Frame<'a>>,
}

struct Frame<'a> {
    breadcrumb: Option<Node<'a>>,
    column: Column<'a>,
}

impl<'a> AlgorithmX<'a> {
    fn new(dlx: Dlx<'a>, num_rows: usize) -> Self {
        AlgorithmX {
            dlx,
            init: true,
            select_flags: BitVec::repeat(false, num_rows),
            stack: Vec::new(),
        }
    }

    fn select(&mut self, node: Node<'a>) {
        self.select_flags.set(node.ix(), true);
        self.dlx.cover(node);
    }

    fn unselect(&mut self, node: Node<'a>) {
        self.dlx.uncover(node);
        self.select_flags.set(node.ix(), false);
    }
}

impl<'a> Iterator for AlgorithmX<'a> {
    type Item = BitVec<Cell<usize>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.init {
            self.init = false;

            let Some(column) = self.dlx.min_size_col() else {
                return Some(self.select_flags.clone());
            };
            self.stack.push(Frame {
                breadcrumb: None,
                column,
            });
        }

        while let Some(frame) = self.stack.last_mut() {
            let Some(node) = frame.column.next() else {
                // backtrack if there's no node to explore in `column`
                if let Some(node) = frame.breadcrumb {
                    self.unselect(node);
                }
                self.stack.pop();
                continue
            };

            self.select(node);

            if let Some(column) = self.dlx.min_size_col() {
                self.stack.push(Frame {
                    breadcrumb: Some(node),
                    column,
                });
            } else {
                // if there's no column to choose, i.e., the DLX is empty, we've found a solution
                let select_flags = self.select_flags.clone();
                self.unselect(node);
                return Some(select_flags);
            }
        }

        None
    }
}
