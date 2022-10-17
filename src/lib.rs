use bit_set::BitSet;
use paste::paste;
use rustc_hash::FxHashMap;
use std::cell::Cell;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::ControlFlow;
use typed_arena::Arena;

pub fn solve<L, T, S>(subsets: impl IntoIterator<Item = (L, HashSet<T, S>)>) -> Option<Vec<L>>
where
    T: Hash + Eq,
{
    let arena = NodeArena::new();
    let mut builder = EcpBuilder::new(&arena);

    for (label, subset) in subsets {
        builder.add_subset(label, subset);
    }

    builder.build().solve()
}

struct EcpBuilder<'a, L, T> {
    headers: FxHashMap<T, Node<'a>>,
    arena: &'a NodeArena<'a>,
    root: Node<'a>,
    labels: Vec<L>,
}

impl<'a, L, T> EcpBuilder<'a, L, T> {
    fn new(arena: &'a NodeArena<'a>) -> Self {
        EcpBuilder {
            headers: FxHashMap::default(),
            arena,
            root: arena.alloc_header(),
            labels: Vec::new(),
        }
    }

    #[inline]
    fn build(self) -> Ecp<'a, L> {
        Ecp {
            dlx: Dlx::new(self.root),
            labels: self.labels,
        }
    }
}

impl<'a, L, T> EcpBuilder<'a, L, T>
where
    T: Hash + Eq,
{
    fn add_subset<S>(&mut self, label: L, subset: HashSet<T, S>) {
        self.labels.push(label);
        let row_ix = self.labels.len() - 1;
        let mut row_header: Option<Node> = None;

        for elem in subset {
            let node = self.arena.alloc(row_ix);
            if let Some(row_header) = row_header {
                row_header.insert_left(node);
            } else {
                row_header = Some(node);
            }

            let header = *self.headers.entry(elem).or_insert_with(|| {
                let header = self.arena.alloc_header();
                self.root.insert_left(header);
                header
            });
            header.inc_size();
            node.set_header(header);
            header.insert_up(node);
        }
    }
}

struct Ecp<'a, L> {
    dlx: Dlx<'a>,
    labels: Vec<L>,
}

impl<'a, L> Ecp<'a, L> {
    #[inline]
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

struct Dlx<'a> {
    root: Node<'a>,
}

impl<'a> Dlx<'a> {
    #[inline]
    fn new(root: Node<'a>) -> Dlx<'a> {
        Dlx { root }
    }

    #[inline]
    fn is_solved(&self) -> bool {
        self.root == self.root.right()
    }

    #[inline]
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
            .then(move || label_indices)
    }
}

struct NodeArena<'a>(Arena<NodeData<'a>>);

impl<'a> NodeArena<'a> {
    #[inline]
    fn new() -> Self {
        NodeArena(Arena::new())
    }

    fn alloc_header(&'a self) -> Node<'a> {
        let node = Node(self.0.alloc(NodeData {
            up: Cell::new(None),
            down: Cell::new(None),
            left: Cell::new(None),
            right: Cell::new(None),
            header: Cell::new(None),
            size_or_ix: Cell::new(0),
        }));
        node.set_up(node);
        node.set_down(node);
        node.set_left(node);
        node.set_right(node);
        node.set_header(node);
        node
    }

    fn alloc(&'a self, row_ix: usize) -> Node<'a> {
        let node = Node(self.0.alloc(NodeData {
            up: Cell::new(None),
            down: Cell::new(None),
            left: Cell::new(None),
            right: Cell::new(None),
            header: Cell::new(None),
            size_or_ix: Cell::new(row_ix),
        }));
        node.set_up(node);
        node.set_down(node);
        node.set_left(node);
        node.set_right(node);
        node.set_header(node);
        node
    }
}

#[derive(Clone, Copy)]
struct Node<'a>(&'a NodeData<'a>);

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node<'a>) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

#[derive(Clone)]
struct NodeData<'a> {
    up: Cell<Option<Node<'a>>>,
    down: Cell<Option<Node<'a>>>,
    left: Cell<Option<Node<'a>>>,
    right: Cell<Option<Node<'a>>>,
    header: Cell<Option<Node<'a>>>,
    // size: the number of nodes in a column (when a node is a column header)
    // ix: the row index (otherwise)
    size_or_ix: Cell<usize>,
}

macro_rules! define_node_get_set {
    ($field:ident) => {
        paste! {
            impl<'a> Node<'a> {
                #[inline]
                fn $field(&self) -> Node<'a> {
                    self.0.$field.get().unwrap()
                }

                #[inline]
                fn [<set_ $field>](&self, node: Node<'a>) {
                    self.0.$field.set(Some(node));
                }
            }
        }
    };
}

define_node_get_set! { up }
define_node_get_set! { down }
define_node_get_set! { left }
define_node_get_set! { right }
define_node_get_set! { header }

impl<'a> Node<'a> {
    #[inline]
    fn size(&self) -> usize {
        self.0.size_or_ix.get()
    }

    #[inline]
    fn inc_size(&self) {
        let size = &self.0.size_or_ix;
        size.set(size.get() + 1);
    }

    #[inline]
    fn dec_size(&self) {
        let size = &self.0.size_or_ix;
        size.set(size.get() - 1);
    }

    #[inline]
    fn ix(&self) -> usize {
        self.0.size_or_ix.get()
    }

    fn insert_up(&self, node: Node<'a>) {
        node.set_down(*self);
        node.set_up(self.up());
        self.up().set_down(node);
        self.set_up(node);
    }

    fn insert_left(&self, node: Node<'a>) {
        node.set_right(*self);
        node.set_left(self.left());
        self.left().set_right(node);
        self.set_left(node);
    }

    fn unlink_ud(&self) {
        self.up().set_down(self.down());
        self.down().set_up(self.up());
        self.header().dec_size();
    }

    fn unlink_lr(&self) {
        self.left().set_right(self.right());
        self.right().set_left(self.left());
    }

    fn relink_ud(&self) {
        self.up().set_down(*self);
        self.down().set_up(*self);
        self.header().inc_size();
    }

    fn relink_lr(&self) {
        self.left().set_right(*self);
        self.right().set_left(*self);
    }
}

macro_rules! define_node_iterator {
    ($dir:ident) => {
        paste! {
            struct [<Iter $dir:camel>]<'a> {
                start: Node<'a>,
                next: Option<Node<'a>>,
            }

            impl<'a> Iterator for [<Iter $dir:camel>]<'a> {
                type Item = Node<'a>;

                fn next(&mut self) -> Option<Self::Item> {
                    let node = self.next?;
                    let next = node.$dir();
                    self.next = (next != self.start).then(|| next);
                    Some(node)
                }
            }

            impl<'a> Node<'a> {
                #[inline]
                fn [<iter_ $dir>](&self) -> [<Iter $dir:camel>]<'a> {
                    [<Iter $dir:camel>] {
                        start: *self,
                        next: Some(*self),
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
