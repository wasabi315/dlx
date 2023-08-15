use super::node::{Node, NodeArena};
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use std::hash::Hash;

pub(crate) struct Problem<L, T> {
    headers: FxHashMap<T, Node>,
    pub(crate) arena: NodeArena,
    pub(crate) root: Node,
    pub(crate) labels: Vec<L>,
}

impl<L, T> Problem<L, T> {
    pub(crate) fn new() -> Self {
        let mut arena = NodeArena::new();
        let root = arena.alloc_header();
        Problem {
            headers: FxHashMap::default(),
            arena,
            root,
            labels: Vec::new(),
        }
    }
}

impl<L, T> Problem<L, T>
where
    T: Hash + Eq,
{
    pub(crate) fn add_subset<S>(&mut self, label: L, subset: HashSet<T, S>) {
        self.labels.push(label);
        let row_ix = self.labels.len() - 1;
        let mut row_header: Option<Node> = None;

        for elem in subset {
            let col_header = *self.headers.entry(elem).or_insert_with(|| {
                let header = self.arena.alloc_header();
                self.root.insert_left(header, &mut self.arena);
                header
            });
            let node = self.arena.alloc(col_header, row_ix);

            if let Some(row_header) = row_header {
                row_header.insert_left(node, &mut self.arena);
            } else {
                row_header = Some(node);
            }
            col_header.insert_up(node, &mut self.arena);
            col_header.inc_size(&mut self.arena);
        }
    }
}
