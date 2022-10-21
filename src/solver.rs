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
        let label_indices = self.dlx.solve()?;
        let mut i = 0;
        self.labels.retain(|_| {
            i += 1;
            label_indices.contains(i - 1)
        });
        Some(self.labels)
    }
}

impl<'a, L: Clone> Solver<'a, L> {
    pub(crate) fn solutions(self) -> Solutions<'a, L> {
        Solutions::new(self)
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

    fn solve(self) -> Option<BitSet> {
        let mut label_indices = BitSet::new();
        let mut stack: Vec<(Option<Node>, Skip<IterDown>)> = Vec::new();

        if self.is_solved() {
            return Some(label_indices);
        }
        let header = self.min_size_col().unwrap();
        stack.push((None, header.iter_down().skip(1)));

        loop {
            let (selected_node, candidate_rows) = stack.last_mut().unwrap();

            if let Some(node) = candidate_rows.next() {
                label_indices.insert(node.ix());
                self.cover(node);

                if self.is_solved() {
                    return Some(label_indices);
                }
                let header = self.min_size_col().unwrap();
                stack.push((Some(node), header.iter_down().skip(1)));

                continue;
            }

            if let Some(node) = selected_node {
                self.uncover(*node);
                label_indices.remove(node.ix());
                stack.pop();

                continue;
            }

            return None;
        }
    }
}

pub(crate) struct Solutions<'a, L> {
    dlx: Dlx<'a>,
    running: bool,
    indices: BitSet,
    stack: Vec<(Option<Node<'a>>, Skip<IterDown<'a>>)>,
    labels: Vec<L>,
}

impl<'a, L: Clone> Solutions<'a, L> {
    fn new(solver: Solver<'a, L>) -> Self {
        Solutions {
            dlx: solver.dlx,
            running: false,
            indices: BitSet::new(),
            stack: Vec::new(),
            labels: solver.labels,
        }
    }
}

impl<'a, L: Clone> Iterator for Solutions<'a, L> {
    type Item = Vec<L>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.running {
            self.running = true;

            if self.dlx.is_solved() {
                return Some(Vec::new());
            }
            let header = self.dlx.min_size_col().unwrap();
            self.stack.push((None, header.iter_down().skip(1)));
        }

        loop {
            let (selected_node, candidate_rows) = self.stack.last_mut().unwrap();

            if let Some(node) = candidate_rows.next() {
                self.indices.insert(node.ix());
                self.dlx.cover(node);

                if self.dlx.is_solved() {
                    let solution = self
                        .labels
                        .iter()
                        .enumerate()
                        .filter_map(|(i, label)| self.indices.contains(i).then(|| label.clone()))
                        .collect();
                    self.dlx.uncover(node);
                    self.indices.remove(node.ix());
                    return Some(solution);
                }
                let header = self.dlx.min_size_col().unwrap();
                self.stack.push((Some(node), header.iter_down().skip(1)));

                continue;
            }

            if let Some(node) = selected_node {
                self.dlx.uncover(*node);
                self.indices.remove(node.ix());
                self.stack.pop();

                continue;
            }

            return None;
        }
    }
}
