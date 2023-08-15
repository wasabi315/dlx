use crate::node::NodeArena;

use super::node::Node;
use super::Problem;
use bit_set::BitSet;
use std::ops::ControlFlow;

pub(crate) struct Solver<L> {
    dlx: Dlx,
    labels: Vec<L>,
}

impl<L> Solver<L> {
    pub(crate) fn new<T>(problem: Problem<L, T>) -> Self {
        Solver {
            dlx: Dlx {
                arena: problem.arena,
                root: problem.root,
            },
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

struct Dlx {
    arena: NodeArena,
    root: Node,
}

impl Dlx {
    fn is_solved(&self) -> bool {
        self.root == self.root.right(&self.arena)
    }

    fn min_size_col(&self) -> Option<Node> {
        let headers = self.root.iter_right(&self.arena).skip(1);
        headers.min_by_key(|header| header.size(&self.arena))
    }

    fn cover(&self, selected_node: Node) {
        for node in selected_node.iter_right(&self.arena) {
            let header = node.header(&self.arena);
            header.unlink_lr(&self.arena);

            for col_node in node
                .iter_down(&self.arena)
                .skip(1)
                .filter(|node| node != &header)
            {
                for row_node in col_node.iter_right(&self.arena).skip(1) {
                    row_node.unlink_ud(&self.arena);
                }
            }
        }
    }

    fn uncover(&self, selected_node: Node) {
        for node in selected_node.left(&self.arena).iter_left(&self.arena) {
            let header = node.header(&self.arena);
            header.relink_lr(&self.arena);

            for col_node in node
                .iter_up(&self.arena)
                .skip(1)
                .filter(|node| node != &header)
            {
                for row_node in col_node.iter_left(&self.arena).skip(1) {
                    row_node.relink_ud(&self.arena);
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

            for node in header.iter_down(&dlx.arena).skip(1) {
                let ix = node.ix(&dlx.arena);
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
