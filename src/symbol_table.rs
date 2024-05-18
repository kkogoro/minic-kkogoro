//! 符号表

use std::collections::HashMap;

use crate::ast::FuncType;

#[derive(Debug, Clone)] //移除了copy
pub struct VarInfoBase {}
#[derive(Debug, Clone)]
pub struct FuncInfoBase {
    pub ret_type: FuncType,
}
impl FuncInfoBase {
    pub fn new(ret_type: FuncType) -> Self {
        FuncInfoBase { ret_type }
    }
}
#[derive(Debug, Clone)]
pub struct ArrayInfoBase {
    pub dims: Vec<i32>,
}
#[derive(Debug, Clone)]
pub enum SymbolInfo {
    Const(i32),
    Var(VarInfoBase),
    Func(FuncInfoBase),
    Array(ArrayInfoBase),
}
impl VarInfoBase {
    pub fn new() -> Self {
        VarInfoBase {}
    }
}
impl ArrayInfoBase {
    pub fn new() -> Self {
        ArrayInfoBase { dims: Vec::new() }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<String, SymbolInfo>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: SymbolInfo) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&SymbolInfo> {
        self.table.get(key)
    }
}
