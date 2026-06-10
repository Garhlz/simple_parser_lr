use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSet<T> {
    items: BTreeSet<T>,
}

impl<T> ItemSet<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> bool {
        self.items.insert(item)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}
