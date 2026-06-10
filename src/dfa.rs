use crate::symbol::Symbol;
use std::collections::BTreeMap;

pub struct Dfa<S> {
    pub states: Vec<S>,
    pub transitions: BTreeMap<(usize, Symbol), usize>,
}
