use crate::node::{Column, Node};

pub(crate) struct Dlx<'a> {
    root: Node<'a>,
}

impl<'a> Dlx<'a> {
    pub(crate) fn new(root: Node<'a>) -> Self {
        Dlx { root }
    }

    pub(crate) fn min_size_col(&self) -> Option<Column<'a>> {
        let headers = self.root.row().skip(1);
        let header = headers.min_by_key(|header| header.size())?;
        let mut column = header.column();
        column.next(); // skip header
        Some(column)
    }

    pub(crate) fn cover(&self, selected_node: Node<'a>) {
        for node in selected_node.row() {
            let header = node.header();
            header.unlink_lr();

            for col_node in node.column().skip(1).filter(|node| node != &header) {
                for row_node in col_node.row().skip(1) {
                    row_node.unlink_ud();
                }
            }
        }
    }

    pub(crate) fn uncover(&self, selected_node: Node<'a>) {
        for node in selected_node.left().row_rev() {
            let header = node.header();
            header.relink_lr();

            for col_node in node.column_rev().skip(1).filter(|node| node != &header) {
                for row_node in col_node.row_rev().skip(1) {
                    row_node.relink_ud();
                }
            }
        }
    }
}
