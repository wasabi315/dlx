use id_arena::{Arena, Id};
use paste::paste;
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Node(Id<NodeData>);

struct NodeData {
    up: Cell<Node>,
    down: Cell<Node>,
    left: Cell<Node>,
    right: Cell<Node>,
    // None if being a column header, otherwise Some(its column header)
    header: Option<Node>,
    // the number of nodes in the column if being a column header, otherwise the row index
    size_or_ix: Cell<usize>,
}

#[derive(Default)]
pub(crate) struct NodeArena(Arena<NodeData>);

impl NodeArena {
    pub(crate) fn new() -> Self {
        NodeArena(Arena::new())
    }

    pub(crate) fn alloc_header(&mut self) -> Node {
        Node(self.0.alloc_with_id(|id| {
            let id = Node(id);
            NodeData {
                up: Cell::new(id),
                down: Cell::new(id),
                left: Cell::new(id),
                right: Cell::new(id),
                header: None,
                size_or_ix: Cell::new(0),
            }
        }))
    }

    pub(crate) fn alloc(&mut self, header: Node, row_ix: usize) -> Node {
        Node(self.0.alloc_with_id(|id| {
            let id = Node(id);
            NodeData {
                up: Cell::new(id),
                down: Cell::new(id),
                left: Cell::new(id),
                right: Cell::new(id),
                header: Some(header),
                size_or_ix: Cell::new(row_ix),
            }
        }))
    }
}

macro_rules! define_node_accessors {
    ($field:ident) => {
        paste! {
            impl Node {
                pub(crate) fn $field(&self, arena: &NodeArena) -> Node {
                    arena.0[self.0].$field.get()
                }

                pub(crate) fn [<set_ $field>](&self, node: Node, arena: &NodeArena) {
                    arena.0[self.0].$field.set(node)
                }
            }
        }
    };
}

define_node_accessors! { up }
define_node_accessors! { down }
define_node_accessors! { left }
define_node_accessors! { right }

impl Node {
    fn is_header(&self, arena: &NodeArena) -> bool {
        arena.0[self.0].header.is_none()
    }

    pub(crate) fn header(&self, arena: &NodeArena) -> Node {
        debug_assert!(!self.is_header(arena));
        arena.0[self.0].header.unwrap()
    }

    pub(crate) fn size(&self, arena: &NodeArena) -> usize {
        debug_assert!(self.is_header(arena));
        arena.0[self.0].size_or_ix.get()
    }

    pub(crate) fn inc_size(&self, arena: &NodeArena) {
        debug_assert!(self.is_header(arena));
        let size = &arena.0[self.0].size_or_ix;
        size.set(size.get() + 1);
    }

    pub(crate) fn dec_size(&self, arena: &NodeArena) {
        debug_assert!(self.is_header(arena));
        let size = &arena.0[self.0].size_or_ix;
        size.set(size.get() - 1);
    }

    pub(crate) fn ix(&self, arena: &NodeArena) -> usize {
        debug_assert!(!self.is_header(arena));
        arena.0[self.0].size_or_ix.get()
    }

    pub(crate) fn insert_up(&self, node: Node, arena: &NodeArena) {
        node.set_down(*self, arena);
        node.set_up(self.up(arena), arena);
        self.up(arena).set_down(node, arena);
        self.set_up(node, arena);
    }

    pub(crate) fn insert_left(&self, node: Node, arena: &NodeArena) {
        node.set_right(*self, arena);
        node.set_left(self.left(arena), arena);
        self.left(arena).set_right(node, arena);
        self.set_left(node, arena);
    }

    pub(crate) fn unlink_ud(&self, arena: &NodeArena) {
        self.up(arena).set_down(self.down(arena), arena);
        self.down(arena).set_up(self.up(arena), arena);
        self.header(arena).dec_size(arena);
    }

    pub(crate) fn unlink_lr(&self, arena: &NodeArena) {
        self.left(arena).set_right(self.right(arena), arena);
        self.right(arena).set_left(self.left(arena), arena);
    }

    pub(crate) fn relink_ud(&self, arena: &NodeArena) {
        self.up(arena).set_down(*self, arena);
        self.down(arena).set_up(*self, arena);
        self.header(arena).inc_size(arena);
    }

    pub(crate) fn relink_lr(&self, arena: &NodeArena) {
        self.left(arena).set_right(*self, arena);
        self.right(arena).set_left(*self, arena);
    }
}

macro_rules! define_node_iterator {
    ($dir:ident) => {
        paste! {
            impl Node {
                pub(crate) fn [<iter_ $dir>]<'a>(&'a self, arena: &'a NodeArena) -> impl Iterator<Item = Node> + 'a {
                    struct Iter<'a> {
                        arena: &'a NodeArena,
                        start: Node,
                        next: Option<Node>,
                    }

                    impl<'a> Iterator for Iter<'a> {
                        type Item = Node;

                        fn next(&mut self) -> Option<Self::Item> {
                            let node = self.next?;
                            let next = node.$dir(self.arena);
                            self.next = (next != self.start).then_some(next);
                            Some(node)
                        }
                    }

                    Iter {
                        arena,
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
