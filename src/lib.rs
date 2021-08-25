use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use typed_arena::Arena;

pub fn solve<L>(subsets: Vec<(L, Vec<usize>)>) -> Option<Vec<L>> {
    let (mut labels, subsets): (Vec<Option<L>>, Vec<Vec<usize>>) = subsets
        .into_iter()
        .map(|(label, subset)| (Some(label), subset))
        .unzip();

    let arena = Arena::new();
    let dlx = Dlx::new(&arena, subsets);

    let solution = dlx.solve();

    solution.map(|indices| {
        indices
            .into_iter()
            .map(|i| labels.get_mut(i).unwrap().take().unwrap())
            .collect()
    })
}

struct Dlx<'a>(Node<'a>);

impl<'a> Dlx<'a> {
    fn new(arena: &'a NodeArena<'a>, subsets: Vec<Vec<usize>>) -> Self {
        fn append_row<'a>(row_header: Node<'a>, node: Node<'a>) {
            *node.right_mut() = row_header;
            *node.left_mut() = row_header.left();
            *row_header.left().right_mut() = node;
            *row_header.left_mut() = node;
        }

        fn append_column<'a>(col_header: Node<'a>, node: Node<'a>) {
            col_header.borrow_mut().size_or_ix += 1;
            *node.header_mut() = col_header;
            *node.down_mut() = col_header;
            *node.up_mut() = col_header.up();
            *col_header.up().down_mut() = node;
            *col_header.up_mut() = node;
        }

        let n_col = subsets.iter().flatten().max().unwrap_or(&0) + 1;

        let head = Node::new_header(arena);
        let mut col_headers = Vec::new();

        for _ in 0..n_col {
            let col_header = Node::new_header(arena);
            append_row(head, col_header);
            col_headers.push(col_header);
        }

        for (row_ix, row) in subsets.iter().enumerate() {
            if row.is_empty() {
                continue;
            }

            let row_header = Node::new(arena, row_ix);
            let col_header = col_headers[row[0]];
            append_column(col_header, row_header);

            for col_ix in row[1..].iter().copied() {
                let node = Node::new(arena, row_ix);
                let col_header = col_headers[col_ix];
                append_column(col_header, node);
                append_row(row_header, node);
            }
        }

        Dlx(head)
    }

    fn is_empty(&self) -> bool {
        self.0 == self.0.right()
    }

    fn min_size_col(&self) -> (Node<'a>, usize) {
        self.0
            .row_right()
            .skip(1)
            .map(|node| (node, node.borrow().size_or_ix))
            .min_by_key(|(_, col_size)| *col_size)
            .unwrap()
    }

    fn cover(&self, selected_node: Node<'a>) {
        for node in selected_node.row_right() {
            let header = node.header();
            *header.left().right_mut() = header.right();
            *header.right().left_mut() = header.left();

            for col_node in node.column_down().skip(1).filter(|node| node != &header) {
                for row_node in col_node.row_right().skip(1) {
                    *row_node.up().down_mut() = row_node.down();
                    *row_node.down().up_mut() = row_node.up();
                    row_node.header().borrow_mut().size_or_ix -= 1;
                }
            }
        }
    }

    fn uncover(&self, selected_node: Node<'a>) {
        for node in selected_node.row_left() {
            let header = node.header();
            *header.left().right_mut() = header;
            *header.right().left_mut() = header;

            for col_node in node.column_up().skip(1).filter(|node| node != &header) {
                for row_node in col_node.row_left().skip(1) {
                    *row_node.up().down_mut() = row_node;
                    *row_node.down().up_mut() = row_node;
                    row_node.header().borrow_mut().size_or_ix += 1;
                }
            }
        }
    }

    fn solve(&self) -> Option<Vec<usize>> {
        enum Status {
            SolutionFound,
            Continue,
        }

        fn solve_sub(dlx: &Dlx, indices: &mut Vec<usize>) -> Status {
            if dlx.is_empty() {
                return Status::SolutionFound;
            }

            let (header, col_size) = dlx.min_size_col();

            if col_size == 0 {
                return Status::Continue;
            }

            for node in header.column_down().skip(1) {
                indices.push(node.borrow().size_or_ix);

                dlx.cover(node);
                if let Status::SolutionFound = solve_sub(dlx, indices) {
                    return Status::SolutionFound;
                }
                dlx.uncover(node);

                indices.pop().unwrap();
            }

            Status::Continue
        }

        let mut indices = Vec::new();
        let status = solve_sub(self, &mut indices);
        if let Status::SolutionFound = status {
            Some(indices)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Node<'a>(&'a RefCell<NodeData<'a>>);

type NodeArena<'a> = Arena<RefCell<NodeData<'a>>>;

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

impl fmt::Debug for NodeData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NodeData {{ up: {:p}, down: {:p}, left: {:p}, right: {:p}, header: {:p}, size_or_ix: {} }}",
            self.up.unwrap().0,
            self.down.unwrap().0,
            self.left.unwrap().0,
            self.right.unwrap().0,
            self.header.unwrap().0,
            self.size_or_ix,
        )
    }
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
        node.borrow_mut().up = Some(node);
        node.borrow_mut().down = Some(node);
        node.borrow_mut().left = Some(node);
        node.borrow_mut().right = Some(node);
        node.borrow_mut().header = Some(node);
        node
    }

    fn borrow(&self) -> Ref<'a, NodeData<'a>> {
        self.0.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'a, NodeData<'a>> {
        self.0.borrow_mut()
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

    fn row_right(&self) -> RowRight<'a> {
        RowRight {
            start: *self,
            next: Some(*self),
        }
    }

    fn row_left(&self) -> RowLeft<'a> {
        RowLeft {
            start: *self,
            next: Some(*self),
        }
    }

    fn column_down(&self) -> ColumnDown<'a> {
        ColumnDown {
            start: *self,
            next: Some(*self),
        }
    }

    fn column_up(&self) -> ColumnUp<'a> {
        ColumnUp {
            start: *self,
            next: Some(*self),
        }
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node<'a>) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

struct RowRight<'a> {
    start: Node<'a>,
    next: Option<Node<'a>>,
}

impl<'a> Iterator for RowRight<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            let next = node.right();
            self.next = if next == self.start { None } else { Some(next) };
            node
        })
    }
}

struct RowLeft<'a> {
    start: Node<'a>,
    next: Option<Node<'a>>,
}

impl<'a> Iterator for RowLeft<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            let next = node.left();
            self.next = if next == self.start { None } else { Some(next) };
            node
        })
    }
}

struct ColumnDown<'a> {
    start: Node<'a>,
    next: Option<Node<'a>>,
}

impl<'a> Iterator for ColumnDown<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            let next = node.down();
            self.next = if next == self.start { None } else { Some(next) };
            node
        })
    }
}

struct ColumnUp<'a> {
    start: Node<'a>,
    next: Option<Node<'a>>,
}

impl<'a> Iterator for ColumnUp<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            let next = node.up();
            self.next = if next == self.start { None } else { Some(next) };
            node
        })
    }
}

#[cfg(test)]
mod test {
    use super::solve;

    #[test]
    fn test1() {
        let subsets = vec![
            ("A", vec![0, 2]),
            ("B", vec![0, 3, 4]),
            ("C", vec![1, 3]),
            ("D", vec![1, 4]),
            ("E", vec![2, 3]),
            ("F", vec![4]),
        ];
        assert_eq!(
            solve(subsets).map(|mut v| {
                v.sort_unstable();
                v
            }),
            Some(vec!["A", "C", "F"])
        );
    }

    #[test]
    fn test2() {
        let subsets = vec![
            ("A", vec![0, 3, 6]),
            ("B", vec![0, 3]),
            ("C", vec![3, 4, 6]),
            ("D", vec![2, 4, 5]),
            ("E", vec![1, 2, 5, 6]),
            ("F", vec![1, 6]),
        ];
        assert_eq!(
            solve(subsets).map(|mut v| {
                v.sort_unstable();
                v
            }),
            Some(vec!["B", "D", "F"])
        );
    }

    #[test]
    fn test3() {
        let subsets = vec![
            ("A", vec![0, 2]),
            ("B", vec![0, 3, 4]),
            ("C", vec![1]),
            ("D", vec![1, 4]),
            ("E", vec![2, 3]),
            ("F", vec![4]),
        ];
        assert_eq!(
            solve(subsets).map(|mut v| {
                v.sort_unstable();
                v
            }),
            None,
        );
    }
}
