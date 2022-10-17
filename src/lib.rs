use bit_set::BitSet;
use id_arena::{Arena, Id};
use paste::paste;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{ControlFlow, Index, IndexMut};

struct NodeArena(Arena<NodeData>);

impl NodeArena {
    #[inline]
    fn new() -> Self {
        NodeArena(Arena::new())
    }

    #[inline]
    fn alloc_header(&mut self) -> Node {
        Node(self.0.alloc_with_id(|id| {
            let node = Node(id);
            NodeData {
                up: node,
                down: node,
                left: node,
                right: node,
                header: node,
                size_or_ix: 0,
            }
        }))
    }

    #[inline]
    fn alloc(&mut self, ix: usize) -> Node {
        Node(self.0.alloc_with_id(|id| {
            let node = Node(id);
            NodeData {
                up: node,
                down: node,
                left: node,
                right: node,
                header: node,
                size_or_ix: ix,
            }
        }))
    }
}

impl Index<Node> for NodeArena {
    type Output = NodeData;

    #[inline]
    fn index(&self, index: Node) -> &Self::Output {
        self.0.index(index.0)
    }
}

impl IndexMut<Node> for NodeArena {
    #[inline]
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        self.0.index_mut(index.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Node(Id<NodeData>);

#[derive(Clone)]
struct NodeData {
    up: Node,
    down: Node,
    left: Node,
    right: Node,
    header: Node,
    // size: the number of nodes in a column (when a node is a column header)
    // ix: the row index (otherwise)
    size_or_ix: usize,
}

impl Node {
    #[inline]
    fn insert_up(self, arena: &mut NodeArena, node: Node) {
        let orig_up = std::mem::replace(&mut arena[self].up, node);
        arena[orig_up].down = node;
        let mut data = &mut arena[node];
        data.up = orig_up;
        data.down = self;
    }

    #[inline]
    fn insert_left(self, arena: &mut NodeArena, node: Node) {
        let orig_left = std::mem::replace(&mut arena[self].left, node);
        arena[orig_left].right = node;
        let mut data = &mut arena[node];
        data.left = orig_left;
        data.right = self;
    }

    fn unlink_ud(self, arena: &mut NodeArena) {
        let NodeData {
            up, down, header, ..
        } = arena[self];
        arena[up].down = down;
        arena[down].up = up;
        arena[header].size_or_ix -= 1;
    }

    fn relink_ud(self, arena: &mut NodeArena) {
        let NodeData {
            up, down, header, ..
        } = arena[self];
        arena[up].down = self;
        arena[down].up = self;
        arena[header].size_or_ix += 1;
    }

    fn unlink_lr(self, arena: &mut NodeArena) {
        let NodeData { left, right, .. } = arena[self];
        arena[left].right = right;
        arena[right].left = left;
    }

    fn relink_lr(self, arena: &mut NodeArena) {
        let NodeData { left, right, .. } = arena[self];
        arena[left].right = self;
        arena[right].left = self;
    }
}

macro_rules! define_node_iterator {
    ($dir:ident) => {
        paste! {
            struct [<Iter $dir:camel>]<'a> {
                arena: &'a RefCell<NodeArena>,
                start: Node,
                next: Option<Node>,
            }

            impl<'a> Iterator for [<Iter $dir:camel>]<'a> {
                type Item = Node;

                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    let node = self.next?;
                    let next = self.arena.borrow()[node].$dir;
                    self.next = (next != self.start).then(|| next);
                    Some(node)
                }
            }

            impl Node {
                #[inline]
                fn [<iter_ $dir>](self, arena: &RefCell<NodeArena>) -> [<Iter $dir:camel>] {
                    [<Iter $dir:camel>] {
                        arena,
                        start: self,
                        next: Some(self),
                    }
                }
            }
        }
    };
}

define_node_iterator! { up }
define_node_iterator! { down }
define_node_iterator! { left }
define_node_iterator! { right }

struct Dlx {
    arena: RefCell<NodeArena>,
    root: Node,
}

impl Dlx {
    #[inline]
    fn is_solved(&self) -> bool {
        self.root == self.arena.borrow()[self.root].right
    }

    #[inline]
    fn min_size_col(&self) -> Option<Node> {
        let headers = self.root.iter_right(&self.arena).skip(1);
        let arena = self.arena.borrow();
        headers.min_by_key(|header| arena[*header].size_or_ix)
    }

    fn cover(&self, selected_node: Node) {
        for node in selected_node.iter_right(&self.arena) {
            let header;
            {
                let mut arena = self.arena.borrow_mut();
                header = arena[node].header;
                header.unlink_lr(&mut arena);
            }

            for col_node in node
                .iter_down(&self.arena)
                .skip(1)
                .filter(|node| node != &header)
            {
                for row_node in col_node.iter_right(&self.arena).skip(1) {
                    row_node.unlink_ud(&mut self.arena.borrow_mut());
                }
            }
        }
    }

    fn uncover(&self, selected_node: Node) {
        let left = self.arena.borrow()[selected_node].left;
        for node in left.iter_left(&self.arena) {
            let header;
            {
                let mut arena = self.arena.borrow_mut();
                header = arena[node].header;
                header.relink_lr(&mut arena);
            }

            for col_node in node
                .iter_up(&self.arena)
                .skip(1)
                .filter(|node| node != &header)
            {
                for row_node in col_node.iter_left(&self.arena).skip(1) {
                    row_node.relink_ud(&mut self.arena.borrow_mut());
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
                let ix = dlx.arena.borrow()[node].size_or_ix;
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
            .then(move || label_indices)
    }
}

struct Ecp<L> {
    dlx: Dlx,
    labels: Vec<L>,
}

impl<L> Ecp<L> {
    fn solve(mut self) -> Option<Vec<L>> {
        let label_indices = self.dlx.solve()?;
        let mut i = 0;
        self.labels.retain(|_| {
            i += 1;
            label_indices.contains(i - 1)
        });
        Some(self.labels)
    }
}

struct EcpBuilder<L, T> {
    arena: NodeArena,
    headers: FxHashMap<T, Node>,
    root: Node,
    labels: Vec<L>,
}

impl<L, T> EcpBuilder<L, T> {
    fn new() -> Self {
        let mut arena = NodeArena::new();
        let root = arena.alloc_header();
        EcpBuilder {
            headers: FxHashMap::default(),
            labels: Vec::new(),
            arena,
            root,
        }
    }

    #[inline]
    fn build(self) -> Ecp<L> {
        Ecp {
            dlx: Dlx {
                arena: RefCell::new(self.arena),
                root: self.root,
            },
            labels: self.labels,
        }
    }
}

impl<L, T> EcpBuilder<L, T>
where
    T: Hash + Eq,
{
    fn add_subset<S>(&mut self, label: L, subset: HashSet<T, S>) {
        self.labels.push(label);
        let row_ix = self.labels.len() - 1;
        let mut row_header: Option<Node> = None;

        let arena = &mut self.arena;
        for elem in subset {
            let node = arena.alloc(row_ix);
            if let Some(row_header) = row_header {
                row_header.insert_left(arena, node);
            } else {
                row_header = Some(node);
            }

            let header = *self.headers.entry(elem).or_insert_with(|| {
                let header = arena.alloc_header();
                self.root.insert_left(arena, header);
                header
            });
            arena[header].size_or_ix += 1;
            arena[node].header = header;
            header.insert_up(arena, node);
        }
    }
}

#[inline]
pub fn solve<L, T, S>(subsets: impl IntoIterator<Item = (L, HashSet<T, S>)>) -> Option<Vec<L>>
where
    T: Hash + Eq,
{
    let mut builder = EcpBuilder::new();

    for (label, subset) in subsets {
        builder.add_subset(label, subset);
    }

    builder.build().solve()
}

#[cfg(test)]
mod test {
    macro_rules! ecp {
        ($($label:expr => {$($elem:expr),*},)*) => {
            [$(($label, ::std::collections::HashSet::from([$($elem),*]))),*]
        };
    }

    #[test]
    fn test1() {
        let ecp = ecp! {
            'A' => {0, 3, 6},
            'B' => {0, 3},
            'C' => {3, 4, 6},
            'D' => {2, 4, 5},
            'E' => {1, 2, 5, 6},
            'F' => {1, 6},
        };
        assert_eq!(super::solve(ecp), Some(vec!['B', 'D', 'F']));
    }

    #[test]
    fn test2() {
        let ecp = ecp! {
           0 => {0, 2}, // *
           1 => {0, 3, 4},
           0 => {1, 3}, // *
           1 => {1, 4},
           0 => {2, 3},
           1 => {4}, // *
        };
        assert_eq!(super::solve(ecp), Some(vec![0, 0, 1]));
    }

    #[test]
    fn test3() {
        let ecp = ecp! {
            () => {0, 2},
            () => {0, 3, 4},
            () => {1},
            () => {1, 4},
            () => {2, 3},
            () => {4},
        };
        assert_eq!(super::solve(ecp), None);
    }
}
