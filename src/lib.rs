use bit_set::BitSet;
use rustc_hash::FxHashMap;
use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::ControlFlow;
use std::ops::DerefMut;
use typed_arena::Arena;

pub fn solve<L, T, S>(subsets: impl IntoIterator<Item = (L, HashSet<T, S>)>) -> Option<Vec<L>>
where
    T: Hash + Eq,
{
    let arena = Arena::new();
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
            root: Node::new_header(arena),
            labels: Vec::new(),
        }
    }

    fn build(self) -> Ecp<'a, L> {
        Ecp {
            root: self.root,
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

        let mut adj_node: Option<Node> = None;
        for elem in subset {
            let node = Node::new(self.arena, row_ix);

            // Link nodes in a same row
            if let Some(adj_node) = adj_node {
                adj_node.hook_right(node);
            }
            adj_node = Some(node);

            // Link node in a same column
            let header = *self.headers.entry(elem).or_insert_with(|| {
                let header = Node::new_header(self.arena);
                self.root.hook_left(header);
                header
            });
            *header.size_mut() += 1;
            *node.header_mut() = header;
            header.hook_up(node);
        }
    }
}

struct Ecp<'a, L> {
    root: Node<'a>,
    labels: Vec<L>,
}

impl<'a, L> Ecp<'a, L> {
    fn is_solved(&self) -> bool {
        self.root == self.root.right()
    }

    fn min_size_col(&self) -> Option<(Node<'a>, usize)> {
        let headers = self.root.iter_right().skip(1);
        headers
            .map(|header| (header, header.size()))
            .min_by(|(_, size0), (_, size1)| size0.cmp(size1))
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

    fn solve(mut self) -> Option<Vec<L>> {
        fn solve_sub<L>(ecp: &Ecp<L>, label_indices: &mut BitSet) -> ControlFlow<()> {
            if ecp.is_solved() {
                return ControlFlow::Break(());
            }

            let (header, col_size) = ecp.min_size_col().unwrap();

            if col_size == 0 {
                return ControlFlow::Continue(());
            }

            for node in header.iter_down().skip(1) {
                let ix = node.ix();
                label_indices.insert(ix);
                ecp.cover(node);

                solve_sub(ecp, label_indices)?;

                ecp.uncover(node);
                label_indices.remove(ix);
            }

            ControlFlow::Continue(())
        }

        let mut label_indices = BitSet::with_capacity(self.labels.len());
        solve_sub(&self, &mut label_indices).is_break().then(|| {
            let mut i = 0;
            self.labels.retain(|_| {
                i += 1;
                label_indices.contains(i - 1)
            });
            self.labels
        })
    }
}

type NodeArena<'a> = Arena<RefCell<NodeData<'a>>>;

#[derive(Clone, Copy)]
struct Node<'a>(&'a RefCell<NodeData<'a>>);

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node<'a>) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

#[derive(Clone, Copy)]
struct NodeData<'a> {
    up: Option<Node<'a>>,
    down: Option<Node<'a>>,
    left: Option<Node<'a>>,
    right: Option<Node<'a>>,
    header: Option<Node<'a>>,
    // size: the number of nodes in a column (when a node is a column header)
    // ix: the row index (otherwise)
    size_or_ix: usize,
}

impl<'a> Node<'a> {
    fn new_header(arena: &'a NodeArena<'a>) -> Self {
        Node::alloc(arena, 0)
    }

    fn new(arena: &'a NodeArena<'a>, row_ix: usize) -> Self {
        Node::alloc(arena, row_ix)
    }

    fn alloc(arena: &'a NodeArena<'a>, size_or_ix: usize) -> Self {
        let node = Node(arena.alloc(RefCell::new(NodeData {
            up: None,
            down: None,
            left: None,
            right: None,
            header: None,
            size_or_ix,
        })));
        node.0.borrow_mut().up = Some(node);
        node.0.borrow_mut().down = Some(node);
        node.0.borrow_mut().left = Some(node);
        node.0.borrow_mut().right = Some(node);
        node.0.borrow_mut().header = Some(node);
        node
    }
}

macro_rules! define_node_getters {
    (get: $acc_name:ident, get_mut: $mut_acc_name:ident) => {
        impl<'a> Node<'a> {
            fn $acc_name(&self) -> Node<'a> {
                self.0.borrow().$acc_name.unwrap()
            }

            fn $mut_acc_name(&self) -> impl DerefMut<Target = Node<'a>> {
                RefMut::map(self.0.borrow_mut(), |node| node.$acc_name.as_mut().unwrap())
            }
        }
    };
}

define_node_getters! { get: up, get_mut: up_mut }
define_node_getters! { get: down, get_mut: down_mut }
define_node_getters! { get: left, get_mut: left_mut }
define_node_getters! { get: right, get_mut: right_mut }
define_node_getters! { get: header, get_mut: header_mut }

impl<'a> Node<'a> {
    fn size(&self) -> usize {
        self.0.borrow().size_or_ix
    }

    fn size_mut(&self) -> impl DerefMut<Target = usize> + 'a {
        RefMut::map(self.0.borrow_mut(), |node| &mut node.size_or_ix)
    }

    fn ix(&self) -> usize {
        self.0.borrow().size_or_ix
    }

    fn hook_up(&self, node: Node<'a>) {
        debug_assert!(self.header() == node.header());

        *node.down_mut() = *self;
        *node.up_mut() = self.up();
        *self.up().down_mut() = node;
        *self.up_mut() = node;
    }

    fn hook_left(&self, node: Node<'a>) {
        *node.right_mut() = *self;
        *node.left_mut() = self.left();
        *self.left().right_mut() = node;
        *self.left_mut() = node;
    }

    fn hook_right(&self, node: Node<'a>) {
        *node.left_mut() = *self;
        *node.right_mut() = self.right();
        *self.right().left_mut() = node;
        *self.right_mut() = node;
    }

    fn unlink_ud(&self) {
        *self.up().down_mut() = self.down();
        *self.down().up_mut() = self.up();
        *self.header().size_mut() -= 1;
    }

    fn unlink_lr(&self) {
        *self.left().right_mut() = self.right();
        *self.right().left_mut() = self.left();
    }

    fn relink_ud(&self) {
        *self.up().down_mut() = *self;
        *self.down().up_mut() = *self;
        *self.header().size_mut() += 1;
    }

    fn relink_lr(&self) {
        *self.left().right_mut() = *self;
        *self.right().left_mut() = *self;
    }
}

macro_rules! define_node_iterator {
    ($iter_method_name:ident: $iter_struct_name:ident { $next:ident }) => {
        struct $iter_struct_name<'a> {
            start: Node<'a>,
            next: Option<Node<'a>>,
        }

        impl<'a> Iterator for $iter_struct_name<'a> {
            type Item = Node<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                self.next.map(|node| {
                    let next = node.$next();
                    self.next = (next != self.start).then(|| next);
                    node
                })
            }
        }

        impl<'a> Node<'a> {
            fn $iter_method_name(&self) -> $iter_struct_name<'a> {
                $iter_struct_name {
                    start: *self,
                    next: Some(*self),
                }
            }
        }
    };
}

define_node_iterator! { iter_up: IterUp { up } }
define_node_iterator! { iter_down: IterDown { down } }
define_node_iterator! { iter_left: IterLeft { left } }
define_node_iterator! { iter_right: IterRight { right } }

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
