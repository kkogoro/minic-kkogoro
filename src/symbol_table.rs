//! 符号表

use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<String, i32>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: i32) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&i32> {
        self.table.get(key)
    }
}
