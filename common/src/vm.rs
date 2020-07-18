use std::collections::HashMap;

pub mod interpret;

use crate::code::{
    Equality,
    Ops,
};

// TODO: rewrite rules

#[derive(Debug)]
pub struct Env {
    forward: HashMap<Ops, Ops>,
    backward: HashMap<Ops, Ops>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            forward: HashMap::new(),
            backward: HashMap::new(),
        }
    }

    pub fn add_equality(&mut self, eq: Equality) {
        let Equality { left, right } = eq;

        self.forward.insert(left.clone(), right.clone());
        self.backward.insert(right, left);
    }

    pub fn lookup(&self, ops: Ops) -> Option<Ops> {
        match self.forward.get(&ops).cloned() {
            Some(o) => Some(o),
            None => match self.backward.get(&ops).cloned() {
                Some(o) => Some(o),
                None => None,
            }
        }
    }
}
