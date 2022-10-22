use super::node::Node;

pub(crate) struct Dlx<'a> {
    root: Node<'a>,
}

impl<'a> Dlx<'a> {
    pub(crate) fn new(root: Node<'a>) -> Self {
        Dlx { root }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.root == self.root.right()
    }

    pub(crate) fn min_size_col(&self) -> Option<Node<'a>> {
        let headers = self.root.iter_right().skip(1);
        headers.min_by_key(|header| header.size())
    }

    pub(crate) fn cover(&self, selected_node: Node<'a>) {
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

    pub(crate) fn uncover(&self, selected_node: Node<'a>) {
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
}
