use bit_set::BitSet;

use crate::dlx::Dlx;
use crate::node::{IterDown, Node};
use crate::Problem;

pub(crate) struct Solver<'a, L> {
    alg: AlgorithmX<'a>,
    labels: Vec<L>,
}

impl<'a, L> Solver<'a, L> {
    pub(crate) fn new<T>(problem: Problem<'a, L, T>) -> Self {
        Solver {
            alg: AlgorithmX::new(Dlx::new(problem.root)),
            labels: problem.labels,
        }
    }

    pub(crate) fn solve(mut self) -> Option<Vec<L>> {
        let solution = self.alg.next()?;
        let mut i = 0;
        self.labels.retain(|_| {
            i += 1;
            solution.contains(i - 1)
        });
        Some(self.labels)
    }

    pub(crate) fn solutions(self) -> Solutions<'a, L> {
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
        let solution = self.alg.next()?;
        let solution = self
            .labels
            .iter()
            .enumerate()
            .filter_map(|(i, label)| solution.contains(i).then_some(label))
            .cloned()
            .collect();
        Some(solution)
    }
}

struct AlgorithmX<'a> {
    dlx: Dlx<'a>,
    init: bool,
    selected_rows: BitSet,
    context: Vec<Context<'a>>,
}

struct Context<'a> {
    selected_row: Option<Node<'a>>,
    column: IterDown<'a>,
}

impl<'a> AlgorithmX<'a> {
    fn new(dlx: Dlx<'a>) -> Self {
        AlgorithmX {
            dlx,
            init: true,
            selected_rows: BitSet::new(),
            context: Vec::new(),
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
        if self.init {
            self.init = false;

            let Some(column) = self.dlx.min_size_col() else {
                // the given DLX is empty thus empty BitSet is the only solution
                return Some(BitSet::new());
            };
            self.context.push(Context {
                selected_row: None,
                column,
            });
        }

        while let Some(ctx) = self.context.last_mut() {
            let Some(row) = ctx.column.next() else {
                // backtrack if there's no node to explore in `column`
                if let Some(row) = ctx.selected_row {
                    self.unselect(row);
                }
                self.context.pop();
                continue;
            };

            self.select(row);

            if let Some(column) = self.dlx.min_size_col() {
                self.context.push(Context {
                    selected_row: Some(row),
                    column,
                });
            } else {
                // if there's no column to choose, i.e., the DLX is empty, we've found a solution
                let solution = self.selected_rows.clone();
                self.unselect(row);
                return Some(solution);
            }
        }

        None
    }
}
