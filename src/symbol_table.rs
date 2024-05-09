//! 符号表

use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct VarTypeBase {}
#[derive(Debug, Copy, Clone)]
pub enum SymbolType {
    Const(i32),
    Var(VarTypeBase),
}

impl VarTypeBase {
    pub fn new() -> Self {
        VarTypeBase {}
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<String, SymbolType>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: SymbolType) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&SymbolType> {
        self.table.get(key)
    }
}
