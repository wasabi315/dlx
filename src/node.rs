use paste::paste;
use std::cell::Cell;
use typed_arena::Arena;

#[derive(Clone, Copy)]
pub(crate) struct Node<'a>(&'a NodeData<'a>);

struct NodeData<'a> {
    up: Cell<Option<Node<'a>>>,
    down: Cell<Option<Node<'a>>>,
    left: Cell<Option<Node<'a>>>,
    right: Cell<Option<Node<'a>>>,
    // None if being a column header, otherwise Some(its column header)
    header: Option<Node<'a>>,
    // the number of nodes in the column if being a column header, otherwise the row index
    size_or_ix: Cell<usize>,
}

#[derive(Default)]
pub(crate) struct NodeArena<'a>(Arena<NodeData<'a>>);

impl<'a> NodeArena<'a> {
    pub(crate) fn new() -> Self {
        NodeArena(Arena::new())
    }

    pub(crate) fn alloc_header(&'a self) -> Node<'a> {
        let node = Node(self.0.alloc(NodeData {
            up: Cell::new(None),
            down: Cell::new(None),
            left: Cell::new(None),
            right: Cell::new(None),
            header: None,
            size_or_ix: Cell::new(0),
        }));
        node.set_up(node);
        node.set_down(node);
        node.set_left(node);
        node.set_right(node);
        node
    }

    pub(crate) fn alloc(&'a self, header: Node<'a>, row_ix: usize) -> Node<'a> {
        let node = Node(self.0.alloc(NodeData {
            up: Cell::new(None),
            down: Cell::new(None),
            left: Cell::new(None),
            right: Cell::new(None),
            header: Some(header),
            size_or_ix: Cell::new(row_ix),
        }));
        node.set_up(node);
        node.set_down(node);
        node.set_left(node);
        node.set_right(node);
        node
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node<'a>) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

macro_rules! define_node_accessors {
    ($field:ident) => {
        paste! {
            impl<'a> Node<'a> {
                pub(crate) fn $field(&self) -> Node<'a> {
                    self.0.$field.get().unwrap()
                }

                pub(crate) fn [<set_ $field>](&self, node: Node<'a>) {
                    self.0.$field.set(Some(node));
                }
            }
        }
    };
}

define_node_accessors! { up }
define_node_accessors! { down }
define_node_accessors! { left }
define_node_accessors! { right }

impl<'a> Node<'a> {
    fn is_header(&self) -> bool {
        self.0.header.is_none()
    }

    pub(crate) fn header(&self) -> Node<'a> {
        debug_assert!(!self.is_header());
        self.0.header.unwrap()
    }

    pub(crate) fn size(&self) -> usize {
        debug_assert!(self.is_header());
        self.0.size_or_ix.get()
    }

    pub(crate) fn inc_size(&self) {
        debug_assert!(self.is_header());
        let size = &self.0.size_or_ix;
        size.set(size.get() + 1);
    }

    pub(crate) fn dec_size(&self) {
        debug_assert!(self.is_header());
        let size = &self.0.size_or_ix;
        size.set(size.get() - 1);
    }

    pub(crate) fn ix(&self) -> usize {
        debug_assert!(!self.is_header());
        self.0.size_or_ix.get()
    }

    pub(crate) fn insert_up(&self, node: Node<'a>) {
        node.set_down(*self);
        node.set_up(self.up());
        self.up().set_down(node);
        self.set_up(node);
    }

    pub(crate) fn insert_left(&self, node: Node<'a>) {
        node.set_right(*self);
        node.set_left(self.left());
        self.left().set_right(node);
        self.set_left(node);
    }

    pub(crate) fn unlink_ud(&self) {
        self.up().set_down(self.down());
        self.down().set_up(self.up());
        self.header().dec_size();
    }

    pub(crate) fn unlink_lr(&self) {
        self.left().set_right(self.right());
        self.right().set_left(self.left());
    }

    pub(crate) fn relink_ud(&self) {
        self.up().set_down(*self);
        self.down().set_up(*self);
        self.header().inc_size();
    }

    pub(crate) fn relink_lr(&self) {
        self.left().set_right(*self);
        self.right().set_left(*self);
    }
}

macro_rules! define_node_iterator {
    ($dir:ident) => {
        paste! {
            pub(crate) struct [<Iter $dir:camel>]<'a> {
                start: Node<'a>,
                next: Option<Node<'a>>,
            }

            impl<'a> Iterator for [<Iter $dir:camel>]<'a> {
                type Item = Node<'a>;

                fn next(&mut self) -> Option<Self::Item> {
                    let node = self.next?;
                    let next = node.$dir();
                    self.next = (next != self.start).then_some(next);
                    Some(node)
                }
            }

            impl<'a> Node<'a> {
                pub(crate) fn [<iter_ $dir>](&self) -> [<Iter $dir:camel>]<'a> {
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
