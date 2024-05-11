use crate::symbol_table::SymbolInfo;
use crate::symbol_table::SymbolTable;

#[derive(Debug)]
pub struct GenerateIrInfo {
    pub now_id: i32,
    pub now_block_id: i32,
    //pub table: SymbolTable,
    pub tables: Vec<SymbolTable>,
    pub block_id: Vec<i32>,
    pub if_id: i32,
    pub and_or_id: i32,          //短路求值块编号
    pub while_id: i32,           //while循环块编号
    pub while_history: Vec<i32>, //从当前到根的循环块编号栈
}

impl GenerateIrInfo {
    pub fn new() -> Self {
        GenerateIrInfo {
            now_id: 0,
            now_block_id: 0,
            //先push一个空的block，编号为0，代表全局?
            //这里的block是对作用域的抽象，和任何符号无关
            tables: vec![SymbolTable::new()],
            block_id: vec![0],
            if_id: 0,
            and_or_id: 0,
            while_id: 0,
            while_history: vec![],
            //table: symbol_table::SymbolTable::new(),
        }
    }
}

///用于记录从符号表查询得到的变量信息和所在block深度
pub struct SymbolReturn {
    pub content: SymbolInfo,
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
        panic!(
            "尝试查询不存在的变量: {}\n当前block_id为{}\n符号表结构为{:#?}",
            key, self.now_block_id, self.tables,
        );
    }
    ///查询当前符号是否为全局符号
    pub fn is_global_symbol(&self, key: &str) -> bool {
        match self.search_symbol(key) {
            Some(SymbolReturn { content, dep }) => match content {
                SymbolInfo::Var(_) => match dep {
                    0 => true,
                    _ => false,
                },
                SymbolInfo::Func(_) => true,
                _ => panic!("尝试查询常量的全局性"),
            },
            None => panic!(
                "尝试查询不存在的符号: {}\n当前block_id为{}\n符号表结构为{:#?}",
                key, self.now_block_id, self.tables,
            ),
        }
    }
    ///得到正确**变量或函数**名
    pub fn get_name(&self, key: &str) -> String {
        match self.search_symbol(key) {
            Some(SymbolReturn { content, dep }) => match content {
                SymbolInfo::Var(_) => match dep {
                    //全局变量符号前面加上GLOBAL_关键字
                    0 => "GLOBAL_".to_string() + key,
                    //局部变量前面加上LOCAL_关键字，后面附上block_id
                    _ => {
                        "LOCAL_".to_string() + key + "_" + &self.block_id[dep as usize].to_string()
                    }
                },
                //函数符号前面加上FUNC_关键字
                SymbolInfo::Func(_) => key.to_string(),
                _ => panic!("尝试查询常量的名称"),
            },
            None => panic!(
                "尝试查询不存在的变量: {}\n当前block_id为{}\n符号表结构为{:#?}",
                key, self.now_block_id, self.tables,
            ),
        }
    }

    //TODO: 检测符号表重复插入，避免符号覆盖

    ///插入符号表
    pub fn insert_symbol(&mut self, key: String, value: SymbolInfo) {
        self.tables.last_mut().unwrap().insert(key, value);

        symbol_table_debug!("插入符号表成功\n表结构为{:#?}", self.tables);
    }
    ///插入全局符号，全局符号表就是tables[0]
    pub fn insert_global_symbol(&mut self, key: String, value: SymbolInfo) {
        self.tables[0].insert(key, value);
        symbol_table_debug!("插入全局符号表成功\n表结构为{:#?}", self.tables);
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

    ///新建一个while循环块
    pub fn push_while(&mut self) {
        self.while_id += 1;
        self.while_history.push(self.while_id);

        while_stack_debug!(
            "新建while循环块: {}\nwhile栈结构为{:#?}",
            self.while_id,
            self.while_history
        );
    }

    ///删除一个while循环块
    pub fn pop_while(&mut self) {
        while_stack_debug!("删除while循环块: {}", self.while_history.last().unwrap(),);

        self.while_history.pop();

        while_stack_debug!("while栈结构为{:#?}", self.while_history);
    }
}
