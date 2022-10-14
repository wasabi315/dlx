use bit_set::BitSet;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::ControlFlow;
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
            let node = Node::new(self.arena, row_ix);
            if let Some(row_header) = row_header {
                row_header.hook_left(node);
            } else {
                row_header = Some(node);
            }

            let header = *self.headers.entry(elem).or_insert_with(|| {
                let header = Node::new_header(self.arena);
                self.root.hook_left(header);
                header
            });
            header.inc_size();
            node.set_header(header);
            header.hook_up(node);
        }
    }
}

struct Ecp<'a, L> {
    dlx: Dlx<'a>,
    labels: Vec<L>,
}

impl<'a, L> Ecp<'a, L> {
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
    #[inline]
    fn new_header(arena: &'a NodeArena<'a>) -> Self {
        Node::alloc(arena, 0)
    }

    #[inline]
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

macro_rules! define_node_get_set {
    (get: $getter:ident, set: $setter:ident) => {
        impl<'a> Node<'a> {
            #[inline]
            fn $getter(&self) -> Node<'a> {
                self.0.borrow().$getter.unwrap()
            }

            #[inline]
            fn $setter(&self, node: Node<'a>) {
                self.0.borrow_mut().$getter = Some(node);
            }
        }
    };
}

define_node_get_set! { get: up, set: set_up }
define_node_get_set! { get: down, set: set_down }
define_node_get_set! { get: left, set: set_left }
define_node_get_set! { get: right, set: set_right }
define_node_get_set! { get: header, set: set_header }

impl<'a> Node<'a> {
    #[inline]
    fn size(&self) -> usize {
        self.0.borrow().size_or_ix
    }

    #[inline]
    fn inc_size(&self) {
        self.0.borrow_mut().size_or_ix += 1;
    }

    #[inline]
    fn dec_size(&self) {
        self.0.borrow_mut().size_or_ix -= 1;
    }

    #[inline]
    fn ix(&self) -> usize {
        self.0.borrow().size_or_ix
    }

    fn hook_up(&self, node: Node<'a>) {
        debug_assert!(self.header() == node.header());

        node.set_down(*self);
        node.set_up(self.up());
        self.up().set_down(node);
        self.set_up(node);
    }

    fn hook_left(&self, node: Node<'a>) {
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
    ($iter_method_name:ident: $iter_struct_name:ident { $next:ident }) => {
        struct $iter_struct_name<'a> {
            start: Node<'a>,
            next: Option<Node<'a>>,
        }

        impl<'a> Iterator for $iter_struct_name<'a> {
            type Item = Node<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let node = self.next?;
                let next = node.$next();
                self.next = (next != self.start).then(|| next);
                Some(node)
            }
        }

        impl<'a> Node<'a> {
            #[inline]
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
