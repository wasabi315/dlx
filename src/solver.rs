use super::node::{IterDown, Node};
use super::Problem;
use bit_set::BitSet;
use std::iter::Skip;

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
        let sol_indices = SolutionsIndices::new(self.dlx).next()?;
        let mut i = 0;
        self.labels.retain(|_| {
            i += 1;
            sol_indices.contains(i - 1)
        });
        Some(self.labels)
    }
}

pub(crate) struct Solutions<'a, L> {
    iter: SolutionsIndices<'a>,
    labels: Vec<L>,
}

impl<'a, L: Clone> Iterator for Solutions<'a, L> {
    type Item = Vec<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let sol_indices = self.iter.next()?;
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
            iter: SolutionsIndices::new(self.dlx),
            labels: self.labels,
        }
    }
}

struct Dlx<'a> {
    root: Node<'a>,
}

impl<'a> Dlx<'a> {
    fn new(root: Node<'a>) -> Self {
        Dlx { root }
    }

    fn is_solved(&self) -> bool {
        self.root == self.root.right()
    }

    fn min_size_col(&self) -> Option<Node<'a>> {
        let headers = self.root.iter_right().skip(1);
        headers.min_by_key(|header| header.size())
    }

    fn cover(&self, selected_node: Node<'a>) {
        for node in selected_node.iter_right() {
            let header = node.header();
            header.unlink_lr();

            for col_node in node.iter_down().skip(1).filter(|node| node != &header) {
                for row_node in col_node.iter_right().skip(1) {
                    row_node.unlink_ud();
                }
            }
        }
    }

    fn uncover(&self, selected_node: Node<'a>) {
        for node in selected_node.left().iter_left() {
            let header = node.header();
            header.relink_lr();

            for col_node in node.iter_up().skip(1).filter(|node| node != &header) {
                for row_node in col_node.iter_left().skip(1) {
                    row_node.relink_ud();
                }
            }
        }
    }
}

struct SolutionsIndices<'a> {
    dlx: Dlx<'a>,
    init: bool,
    indices: BitSet,
    stack: Vec<(Option<Node<'a>>, Skip<IterDown<'a>>)>,
}

impl<'a> SolutionsIndices<'a> {
    fn new(dlx: Dlx<'a>) -> Self {
        SolutionsIndices {
            dlx,
            init: true,
            indices: BitSet::new(),
            stack: Vec::new(),
        }
    }
}

impl<'a> Iterator for SolutionsIndices<'a> {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.init {
            self.init = false;

            if self.dlx.is_solved() {
                return Some(self.indices.clone());
            }
            let header = self.dlx.min_size_col().unwrap();
            self.stack.push((None, header.iter_down().skip(1)));
        }

        while let Some((selected_node, candidate_rows)) = self.stack.last_mut() {
            if let Some(node) = candidate_rows.next() {
                let ix = node.ix();
                self.indices.insert(ix);
                self.dlx.cover(node);

                if self.dlx.is_solved() {
                    let solution = self.indices.clone();
                    self.dlx.uncover(node);
                    self.indices.remove(ix);
                    return Some(solution);
                }
                let header = self.dlx.min_size_col().unwrap();
                self.stack.push((Some(node), header.iter_down().skip(1)));

                continue;
            }

            if let Some(node) = *selected_node {
                self.dlx.uncover(node);
                self.indices.remove(node.ix());
            }
            self.stack.pop();
        }

        None
    }
}
