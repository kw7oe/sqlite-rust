use crate::node::{
    InternalCell, Node, NodeType, LEAF_NODE_LEFT_SPLIT_COUNT, LEAF_NODE_RIGHT_SPLIT_COUNT,
};
use crate::row::Row;
use crate::Cursor;

#[derive(Debug)]
pub struct Tree(Vec<Node>);
impl Tree {
    pub fn new() -> Self {
        Tree(Vec::new())
    }

    pub fn nodes(&self) -> &Vec<Node> {
        &self.0
    }

    pub fn mut_nodes(&mut self) -> &mut Vec<Node> {
        &mut self.0
    }

    pub fn create_new_root(
        &mut self,
        cursor: &Cursor,
        old_node_page_num: usize,
        mut new_node: Node,
    ) {
        println!("--- create_new_root: cursor.page_num: {}", cursor.page_num);
        let old_node = self.0.get_mut(old_node_page_num).unwrap();
        let mut root_node = Node::new(true, NodeType::Internal);
        old_node.is_root = false;

        root_node.num_of_cells += 1;
        root_node.right_child_offset = cursor.page_num as u32 + 2;

        old_node.parent_offset = 0;
        old_node.next_leaf_offset = cursor.page_num as u32 + 2;

        new_node.parent_offset = 0;

        let left_max_key = old_node.get_max_key();
        let cell = InternalCell::new(cursor.page_num as u32 + 1, left_max_key);
        root_node.internal_cells.insert(0, cell);

        self.0.insert(0, root_node);
        self.0.insert(cursor.page_num + 2, new_node);
    }

    pub fn split_and_insert_leaf_node(&mut self, cursor: &Cursor, row: &Row) {
        println!("--- split_and_insert_leaf_node: {}", row.id);
        let last_unused_page_num = self.0.len() as u32;
        let mut right_node = self.0.get_mut(cursor.page_num).unwrap();
        let old_max = right_node.get_max_key();
        right_node.insert(row, cursor);

        let mut left_node = Node::new(false, right_node.node_type);

        for _i in 0..LEAF_NODE_RIGHT_SPLIT_COUNT {
            let cell = right_node.cells.remove(LEAF_NODE_LEFT_SPLIT_COUNT);
            right_node.num_of_cells -= 1;

            left_node.cells.push(cell);
            left_node.num_of_cells += 1;
        }

        if right_node.is_root {
            self.create_new_root(cursor, cursor.page_num, left_node);
        } else {
            println!("--- split leaf node and update parent");
            left_node.next_leaf_offset = cursor.page_num as u32;
            let parent_page_num = right_node.parent_offset as usize;
            let new_max = right_node.get_max_key();

            let parent = &mut self.0[parent_page_num];
            parent.update_internal_key(old_max, new_max);

            self.0.push(left_node);

            self.insert_internal_node(parent_page_num, last_unused_page_num as usize);
            self.maybe_split_internal_node(parent_page_num);
        }
    }

    pub fn maybe_split_internal_node(&mut self, parent_page_num: usize) {
        let max_num_cells_for_internal_node = 3;
        let last_unused_page_num = self.0.len() as u32;
        let node = &mut self.0[parent_page_num];

        if node.num_of_cells > max_num_cells_for_internal_node {
            let split_at_index = node.num_of_cells as usize / 2;

            let mut left_node = Node::new(false, node.node_type);
            let mut right_node = Node::new(false, node.node_type);

            for i in 0..split_at_index {
                let ic = node.internal_cells.remove(0);
                left_node.internal_insert(i, ic);
                left_node.num_of_cells += 1;
            }

            let ic = node.internal_cells.remove(0);
            left_node.right_child_offset = ic.child_pointer();
            left_node.parent_offset = parent_page_num as u32;

            for i in 0..node.internal_cells.len() {
                let ic = node.internal_cells.remove(0);
                right_node.internal_insert(i, ic);
                right_node.num_of_cells += 1;
            }
            right_node.right_child_offset = node.right_child_offset;
            right_node.parent_offset = parent_page_num as u32;

            let ic = InternalCell::new(last_unused_page_num, ic.key());
            node.right_child_offset = last_unused_page_num + 1;
            node.internal_insert(0, ic);
            node.num_of_cells = 1;

            self.0.push(left_node);
            self.update_children_parent_offset(last_unused_page_num);

            self.0.push(right_node);
            self.update_children_parent_offset(last_unused_page_num + 1);
        }
    }

    pub fn update_children_parent_offset(&mut self, page_num: u32) {
        let node = &self.0[page_num as usize];

        let mut child_pointers = vec![node.right_child_offset as usize];
        for cell in &node.internal_cells {
            child_pointers.push(cell.child_pointer() as usize);
        }

        for i in child_pointers {
            let child = &mut self.0[i];
            child.parent_offset = page_num;
        }
    }

    pub fn insert_internal_node(&mut self, parent_page_num: usize, new_child_page_num: usize) {
        let parent_right_child_offset = self.0[parent_page_num].right_child_offset as usize;
        let new_node = &self.0[new_child_page_num];
        let new_child_max_key = new_node.get_max_key();

        let right_child = &self.0[parent_right_child_offset];
        let right_max_key = right_child.get_max_key();

        let parent = &mut self.0[parent_page_num];
        parent.num_of_cells += 1;

        let index = parent.internal_search(new_child_max_key);
        if new_child_max_key > right_max_key {
            println!("--- child max key: {new_child_max_key} > right_max_key: {right_max_key}");
            println!("parent_right_child_offset: {parent_right_child_offset}");
            parent.right_child_offset = new_child_page_num as u32;
            parent.internal_insert(
                index,
                InternalCell::new(parent_right_child_offset as u32, right_max_key),
            );
        } else {
            println!("--- child max key: {new_child_max_key} <= right_max_key: {right_max_key}");
            parent.internal_insert(
                index,
                InternalCell::new(new_child_page_num as u32, new_child_max_key),
            );
        }
    }

    pub fn node_to_string(&self, node: &Node, indent_level: usize) -> String {
        let mut result = String::new();

        if node.node_type == NodeType::Internal {
            for _ in 0..indent_level {
                result += "  ";
            }
            result += &format!("- internal (size {})\n", node.num_of_cells);

            for c in &node.internal_cells {
                let child_index = c.child_pointer() as usize;
                let node = &self.0[child_index];
                result += &self.node_to_string(&node, indent_level + 1);

                for _ in 0..indent_level + 1 {
                    result += "  ";
                }
                result += &format!("- key {}\n", c.key());
            }

            let child_index = node.right_child_offset as usize;
            let node = &self.0[child_index];
            result += &self.node_to_string(&node, indent_level + 1);
        } else if node.node_type == NodeType::Leaf {
            for _ in 0..indent_level {
                result += "  ";
            }

            result += &format!("- leaf (size {})\n", node.num_of_cells);
            for c in &node.cells {
                for _ in 0..indent_level + 1 {
                    result += "  ";
                }
                result += &format!("- {}\n", c.key());
            }
        }

        result
    }

    pub fn to_string(&self) -> String {
        let node = &self.0[0];
        self.node_to_string(node, 0)
    }
}
