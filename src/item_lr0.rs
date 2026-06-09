use std::collections::BTreeSet;
use std::fmt;

use crate::{grammar::Grammar, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Item {
    pub production_id: usize,
    pub dot: usize,
}

impl Item {
    pub fn new(production_id: usize, dot: usize) -> Self {
        Self { production_id, dot }
    }

    pub fn symbol_after_dot<'a>(&self, grammar: &'a Grammar) -> Option<&'a Symbol> {
        grammar.production(self.production_id)?.rhs.get(self.dot)
    }

    pub fn is_complete(&self, grammar: &Grammar) -> bool {
        grammar
            .production(self.production_id)
            .is_some_and(|production| self.dot >= production.rhs.len())
    }

    pub fn advance_dot(&self, grammar: &Grammar) -> Option<Self> {
        if self.is_complete(grammar) {
            return None;
        }

        Some(Self {
            production_id: self.production_id,
            dot: self.dot + 1,
        })
    }

    pub fn format_with_grammar(&self, grammar: &Grammar) -> String {
        match grammar.production(self.production_id) {
            Some(production) => {
                let mut parts = Vec::new();

                if production.rhs.is_empty() {
                    parts.push("·".to_string());
                    parts.push("ε".to_string());
                } else {
                    for (index, symbol) in production.rhs.iter().enumerate() {
                        if index == self.dot {
                            parts.push("·".to_string());
                        }
                        parts.push(symbol.to_string());
                    }

                    if self.dot >= production.rhs.len() {
                        parts.push("·".to_string());
                    }
                }

                format!(
                    "({}) {} -> {}",
                    self.production_id,
                    production.lhs,
                    parts.join(" ")
                )
            }
            None => format!("#{} @ {}", self.production_id, self.dot),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} @ {}", self.production_id, self.dot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    items: BTreeSet<Item>,
}

impl State {
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, item: Item) -> bool {
        self.items.insert(item)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }

    pub fn format_with_grammar(&self, grammar: &Grammar) -> String {
        let mut lines = Vec::new();
        lines.push("{".to_string());
        for item in self.iter() {
            lines.push(format!("  {}", item.format_with_grammar(grammar)));
        }
        lines.push("}".to_string());
        lines.join("\n")
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for item in self.iter() {
            writeln!(f, "  {item}")?;
        }
        write!(f, "}}")
    }
}
