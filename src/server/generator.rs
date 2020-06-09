use std::cell::RefCell;

use crate::types;

pub struct SessionIdGenerator {
    seed: RefCell<types::SessionId>,
}

impl Default for SessionIdGenerator {
    fn default() -> Self {
        Self {
            seed: RefCell::new(1),
        }
    }
}

impl SessionIdGenerator {
    pub fn generate(&self) -> types::SessionId {
        self.seed.replace_with(|&mut old| old + 1)
    }
}