use bit_set::BitSet;
use std::ops::ControlFlow;
use super::node::Node;
use super::Problem;

pub(crate) struct Solver<'a, L> {
    dlx: Dlx<'a>,
    labels: Vec<L>,
}

impl<'a, L> Solver<'a, L> {
    pub(crate) fn new<T>(problem: Problem<'a, L, T>) -> Self {
        Solver {
            dlx: Dlx { root: problem.root },
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

struct Dlx<'a> {
    root: Node<'a>,
}

impl<'a> Dlx<'a> {
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
        fn solve_sub(dlx: &Dlx, label_indices: &mut BitSet) -> ControlFlow<()> {
            if dlx.is_solved() {
                return ControlFlow::Break(());
            }

            let header = dlx.min_size_col().unwrap();

            for node in header.iter_down().skip(1) {
                let ix = node.ix();
                label_indices.insert(ix);
                dlx.cover(node);

                solve_sub(dlx, label_indices)?;

                dlx.uncover(node);
                label_indices.remove(ix);
            }

            ControlFlow::Continue(())
        }

        let mut label_indices = BitSet::new();
        solve_sub(&self, &mut label_indices)
            .is_break()
            .then_some(label_indices)
    }
}
