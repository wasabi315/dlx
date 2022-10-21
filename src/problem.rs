use super::node::{Node, NodeArena};
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use std::hash::Hash;

pub(crate) struct Problem<'a, L, T> {
    headers: FxHashMap<T, Node<'a>>,
    arena: &'a NodeArena<'a>,
    pub(crate) root: Node<'a>,
    pub(crate) labels: Vec<L>,
}

impl<'a, L, T> Problem<'a, L, T> {
    pub(crate) fn new(arena: &'a NodeArena<'a>) -> Self {
        Problem {
            headers: FxHashMap::default(),
            arena,
            root: arena.alloc_header(),
            labels: Vec::new(),
        }
    }
}

impl<'a, L, T> Problem<'a, L, T>
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
                self.root.insert_left(header);
                header
            });
            let node = self.arena.alloc(col_header, row_ix);

            if let Some(row_header) = row_header {
                row_header.insert_left(node);
            } else {
                row_header = Some(node);
            }
            col_header.insert_up(node);
            col_header.inc_size();
        }
    }
}
