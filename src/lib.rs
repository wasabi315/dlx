use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use typed_arena::Arena;

pub fn solve<L, T, I>(subsets: I) -> Option<Vec<L>>
where
    T: Hash + Eq,
    I: IntoIterator<Item = (L, FxHashSet<T>)>,
{
    let arena = Arena::new();
    let mut ecp = Ecp::new(&arena);

    for (label, subset) in subsets {
        ecp.add_subset(label, subset);
    }

    ecp.solve()
}

struct Ecp<'a, L, T> {
    arena: &'a Arena<RefCell<NodeData<'a>>>,
    root: Node<'a>,
    headers: FxHashMap<T, Node<'a>>,
    labels: Vec<L>,
}

impl<'a, L, T> Ecp<'a, L, T>
where
    T: Hash + Eq,
{
    fn new(arena: &'a NodeArena<'a>) -> Self {
        Ecp {
            arena,
            root: Node::new_header(arena),
            headers: FxHashMap::default(),
            labels: Vec::new(),
        }
    }

    fn add_subset(&mut self, label: L, subset: FxHashSet<T>) {
        self.labels.push(label);
        let row_ix = self.labels.len() - 1;

        let nodes: Vec<Node> = (0..subset.len())
            .map(|_| Node::new(self.arena, row_ix))
            .collect();

        // Link nodes in the same row
        for window in nodes.windows(2) {
            window[0].hook_right(window[1]);
        }

        // Link nodes in the same column
        let arena = self.arena;
        let root = self.root;
        for (elem, node) in subset.into_iter().zip(nodes.into_iter()) {
            let header = *self.headers.entry(elem).or_insert_with(|| {
                let header = Node::new_header(arena);
                root.hook_left(header);
                header
            });
            *header.size_mut() += 1;
            *node.header_mut() = header;
            header.hook_up(node);
        }
    }

    fn is_solved(&self) -> bool {
        self.root == self.root.right()
    }

    fn min_size_col(&self) -> Option<(Node<'a>, usize)> {
        let mut headers = self.root.iter_right().skip(1);

        let first = headers.next()?;
        Some(headers.fold((first, first.size()), |min, node| {
            let size = node.size();
            if min.1 > size {
                (node, size)
            } else {
                min
            }
        }))
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

    fn solve_sub(&self, label_indices: &mut FxHashSet<usize>) -> bool {
        if self.is_solved() {
            return true;
        }

        let (header, col_size) = self.min_size_col().unwrap();

        if col_size == 0 {
            return false;
        }

        for node in header.iter_down().skip(1) {
            let ix = node.ix();
            label_indices.insert(ix);
            self.cover(node);

            if self.solve_sub(label_indices) {
                return true;
            }

            self.uncover(node);
            label_indices.remove(&ix);
        }

        false
    }

    fn solve(mut self) -> Option<Vec<L>> {
        let mut label_indices = FxHashSet::default();
        let is_solved = self.solve_sub(&mut label_indices);
        is_solved.then(|| {
            let mut i = 0;
            self.labels.retain(|_| {
                i += 1;
                label_indices.contains(&(i - 1))
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

    fn up(&self) -> Node<'a> {
        self.0.borrow().up.unwrap()
    }

    fn up_mut(&self) -> RefMut<'a, Node<'a>> {
        RefMut::map(self.0.borrow_mut(), |node| node.up.as_mut().unwrap())
    }

    fn down(&self) -> Node<'a> {
        self.0.borrow().down.unwrap()
    }

    fn down_mut(&self) -> RefMut<'a, Node<'a>> {
        RefMut::map(self.0.borrow_mut(), |node| node.down.as_mut().unwrap())
    }

    fn left(&self) -> Node<'a> {
        self.0.borrow().left.unwrap()
    }

    fn left_mut(&self) -> RefMut<'a, Node<'a>> {
        RefMut::map(self.0.borrow_mut(), |node| node.left.as_mut().unwrap())
    }

    fn right(&self) -> Node<'a> {
        self.0.borrow().right.unwrap()
    }

    fn right_mut(&self) -> RefMut<'a, Node<'a>> {
        RefMut::map(self.0.borrow_mut(), |node| node.right.as_mut().unwrap())
    }

    fn header(&self) -> Node<'a> {
        self.0.borrow().header.unwrap()
    }

    fn header_mut(&self) -> RefMut<'a, Node<'a>> {
        RefMut::map(self.0.borrow_mut(), |node| node.header.as_mut().unwrap())
    }

    fn size(&self) -> usize {
        self.0.borrow().size_or_ix
    }

    fn size_mut(&self) -> RefMut<'a, usize> {
        RefMut::map(self.0.borrow_mut(), |node| &mut node.size_or_ix)
    }

    fn ix(&self) -> usize {
        self.0.borrow().size_or_ix
    }

    fn hook_up(&self, node: Node<'a>) {
        #[cfg(debug_assertions)]
        if self.header() != node.header() {
            panic!();
        }

        *node.down_mut() = *self;
        *node.up_mut() = self.up();
        *self.up().down_mut() = node;
        *self.up_mut() = node;
    }

    fn hook_down(&self, node: Node<'a>) {
        #[cfg(debug_assertions)]
        if self.header() != node.header() {
            panic!();
        }

        *node.up_mut() = *self;
        *node.down_mut() = self.down();
        *self.down().up_mut() = node;
        *self.down_mut() = node;
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

    fn iter_up(&self) -> Iter<'a> {
        Iter::new(self, |node| node.up())
    }

    fn iter_down(&self) -> Iter<'a> {
        Iter::new(self, |node| node.down())
    }

    fn iter_left(&self) -> Iter<'a> {
        Iter::new(self, |node| node.left())
    }

    fn iter_right(&self) -> Iter<'a> {
        Iter::new(self, |node| node.right())
    }
}

struct Iter<'a> {
    start: Node<'a>,
    next: Option<Node<'a>>,
    get_next: fn(Node) -> Node,
}

impl<'a> Iter<'a> {
    fn new(node: &Node<'a>, get_next: fn(Node) -> Node) -> Self {
        Iter {
            start: *node,
            next: Some(*node),
            get_next,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            let next = (self.get_next)(node);
            self.next = (next != self.start).then(|| next);
            node
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test1() {
        let subsets = vec![
            ("A", vec![0, 3, 6].into_iter().collect()),
            ("B", vec![0, 3].into_iter().collect()),
            ("C", vec![3, 4, 6].into_iter().collect()),
            ("D", vec![2, 4, 5].into_iter().collect()),
            ("E", vec![1, 2, 5, 6].into_iter().collect()),
            ("F", vec![1, 6].into_iter().collect()),
        ];
        assert_eq!(
            super::solve(subsets).map(|mut v| {
                v.sort_unstable();
                v
            }),
            Some(vec!["B", "D", "F"])
        );
    }

    #[test]
    fn test2() {
        let subsets = vec![
            ("A", vec![0, 2].into_iter().collect()),
            ("B", vec![0, 3, 4].into_iter().collect()),
            ("C", vec![1, 3].into_iter().collect()),
            ("D", vec![1, 4].into_iter().collect()),
            ("E", vec![2, 3].into_iter().collect()),
            ("F", vec![4].into_iter().collect()),
        ];
        assert_eq!(
            super::solve(subsets).map(|mut v| {
                v.sort_unstable();
                v
            }),
            Some(vec!["A", "C", "F"])
        );
    }

    #[test]
    fn test3() {
        let subsets = vec![
            ("A", vec![0, 2].into_iter().collect()),
            ("B", vec![0, 3, 4].into_iter().collect()),
            ("C", vec![1].into_iter().collect()),
            ("D", vec![1, 4].into_iter().collect()),
            ("E", vec![2, 3].into_iter().collect()),
            ("F", vec![4].into_iter().collect()),
        ];
        assert_eq!(
            super::solve(subsets).map(|mut v| {
                v.sort_unstable();
                v
            }),
            None,
        );
    }
}
