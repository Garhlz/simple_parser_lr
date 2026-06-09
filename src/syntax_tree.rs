use crate::symbol::{NonTerminal, Terminal};
use std::fmt;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<usize>,
}

// 节点池用索引引用 children，避免在 reduce 时频繁移动整棵子树。
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    pub nodes: Vec<TreeNode>,
    pub root: Option<usize>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }

    // 终结符节点直接保存最终展示文本，id/num 额外携带词素。
    pub fn add_terminal(&mut self, terminal: Terminal, lexeme: &str) -> usize {
        let label = if terminal == Terminal::Id || terminal == Terminal::Num {
            format!("{terminal}({lexeme})")
        } else {
            terminal.to_string()
        };

        self.push_node(label, Vec::new())
    }

    pub fn add_epsilon(&mut self) -> usize {
        self.push_node("ε".to_string(), Vec::new())
    }

    pub fn add_non_terminal(
        &mut self,
        non_terminal: NonTerminal,
        children: Vec<usize>,
    ) -> usize {
        self.push_node(non_terminal.to_string(), children)
    }

    // 根节点会在 accept 时设置，此前树可能已经构造完但尚未对外可见。
    pub fn set_root(&mut self, root: usize) {
        self.root = Some(root);
    }

    fn push_node(&mut self, label: String, children: Vec<usize>) -> usize {
        self.nodes.push(TreeNode { label, children });
        self.nodes.len() - 1
    }

    fn format_node(&self, node_id: usize, indent: usize, lines: &mut Vec<String>) {
        let node = &self.nodes[node_id];
        lines.push(format!("{}{}", "  ".repeat(indent), node.label));

        for &child_id in &node.children {
            self.format_node(child_id, indent + 1, lines);
        }
    }

    // 统一输出缩进树，便于在终端里直接检查 reduce 结果。
    pub fn format(&self) -> String {
        let Some(root) = self.root else {
            return "(syntax tree unavailable)".to_string();
        };

        let mut lines = Vec::new();
        self.format_node(root, 0, &mut lines);
        lines.join("\n")
    }
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}
