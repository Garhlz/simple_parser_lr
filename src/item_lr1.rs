use crate::{
    grammar::Grammar,
    item_set::ItemSet,
    symbol::{Symbol, Terminal},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemLR1 {
    pub production_id: usize,
    pub dot: usize,
    pub lookahead: Terminal,
}

impl ItemLR1 {
    pub fn new(production_id: usize, dot: usize, lookahead: Terminal) -> Self {
        Self {
            production_id,
            dot,
            lookahead,
        }
    }

    pub fn symbol_after_dot<'a>(&self, grammar: &'a Grammar) -> Option<&'a Symbol> {
        grammar.production(self.production_id)?.rhs.get(self.dot)
    }

    pub fn is_complete(&self, grammar: &Grammar) -> bool {
        grammar
            .production(self.production_id)
            .is_some_and(|production| self.dot == production.rhs.len())
    }

    pub fn advance_dot(&self, grammar: &Grammar) -> Option<Self> {
        if self.is_complete(grammar) {
            return None;
        }

        Some(Self {
            production_id: self.production_id,
            dot: self.dot + 1,
            lookahead: self.lookahead,
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
                    "({}) {} -> {}, {}",
                    self.production_id,
                    production.lhs,
                    parts.join(" "),
                    self.lookahead
                )
            }
            None => format!("#{} @ {}, {}", self.production_id, self.dot, self.lookahead),
        }
    }
}

pub type StateLR1 = ItemSet<ItemLR1>;

impl StateLR1 {
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
