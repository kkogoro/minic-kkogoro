use crate::symbol_table::SymbolTable;
use crate::symbol_table::SymbolType;

#[derive(Debug)]
pub struct GenerateIrInfo {
    pub now_id: i32,
    pub now_block_id: i32,
    //pub table: SymbolTable,
    pub tables: Vec<SymbolTable>,
    pub block_id: Vec<i32>,
    pub if_id: i32,
}

impl GenerateIrInfo {
    pub fn new() -> Self {
        GenerateIrInfo {
            now_id: 0,
            now_block_id: 0,
            //先push一个空的block，编号为0，代表全局?
            tables: vec![SymbolTable::new()],
            block_id: vec![0],
            if_id: 0,
            //table: symbol_table::SymbolTable::new(),
        }
    }
}

///用于记录从符号表查询得到的变量信息和所在block深度
pub struct SymbolReturn {
    pub content: SymbolType,
    pub dep: i32,
}

impl GenerateIrInfo {
    ///查询符号表，返回变量信息和所在block深度
    pub fn search_symbol(&self, key: &str) -> Option<SymbolReturn> {
        symbol_table_debug!("search_symbol: key = {}\n表结构为{:#?}", key, self.tables);

        for (dep, table) in self.tables.iter().enumerate().rev() {
            if let Some(content) = table.get(key) {
                return Some(SymbolReturn {
                    content: *content,
                    dep: dep as i32,
                });
            }
        }
        None
    }
    ///得到正确**变量**名
    pub fn get_name(&self, key: &str) -> Option<String> {
        match self.search_symbol(key) {
            Some(SymbolReturn { content, dep }) => match content {
                SymbolType::Var(_) => {
                    Some(key.to_string() + "_" + &self.block_id[dep as usize].to_string())
                }
                _ => panic!("尝试查询常量的名称"),
            },
            None => panic!(
                "尝试查询不存在的变量: {}\n当前block_id为{}\n符号表结构为{:#?}",
                key, self.now_block_id, self.tables,
            ),
        }
    }
    ///插入符号表
    pub fn insert_symbol(&mut self, key: String, value: SymbolType) {
        self.tables.last_mut().unwrap().insert(key, value);

        symbol_table_debug!("插入符号表成功\n表结构为{:#?}", self.tables);
    }

    ///新建一个block
    pub fn push_block(&mut self) {
        self.now_block_id += 1;
        self.block_id.push(self.now_block_id);
        self.tables.push(SymbolTable::new());

        symbol_table_debug!(
            "新建block: {}\n表结构为{:#?}",
            self.now_block_id,
            self.tables
        );
    }

    ///删除一个block
    pub fn pop_block(&mut self) {
        symbol_table_debug!("删除block: {}", self.block_id.last().unwrap(),);

        self.block_id.pop();
        self.tables.pop();

        symbol_table_debug!("表结构为{:#?}", self.tables);
    }
}
