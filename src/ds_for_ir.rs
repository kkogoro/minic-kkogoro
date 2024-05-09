use crate::symbol_table::SymbolTable;

#[derive(Debug)]
pub struct GenerateIrInfo {
    pub now_id: i32,
    pub const_val: SymbolTable,
}
