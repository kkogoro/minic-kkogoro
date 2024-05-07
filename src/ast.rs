//! 基础的AST定义
use std::fmt;

#[derive(Debug)]
///CompUnit is BaseAST
pub struct CompUnit {
    pub func_def: FuncDef,
}
///为CompUnit结构体实现Display trait
impl fmt::Display for CompUnit {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(_f, "{}", self.func_def)
        // 由于 fmt 方法的返回类型是 fmt::Result，因此我们需要手动返回 Ok(())
    }
}

#[derive(Debug)]
///FuncDef is BaseAST
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}
/// 为 FuncDef 结构体实现 Display trait
impl fmt::Display for FuncDef {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(_f, "fun @{}(): {}", self.ident, self.func_type)?;
        write!(_f, " ")?;
        write!(_f, "{}", self.block)
        // 由于 fmt 方法的返回类型是 fmt::Result，因此我们需要手动返回 Ok(())
    }
}

#[derive(Debug)]
///FuncType is BaseAST
/// 枚举类型，表示函数的返回值类型
pub enum FuncType {
    Int,
}
// 为 FuncType 结构体实现 Display trait
impl fmt::Display for FuncType {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(_f, "i32")
    }
}

#[derive(Debug)]
///Block is BaseAST
pub struct Block {
    pub stmt: Stmt,
}
/// 为 Block 结构体实现 Display trait
impl fmt::Display for Block {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(_f, "{{\n")?;
        write!(_f, "%entry:\n")?;
        write!(_f, "{}", self.stmt)?;
        write!(_f, "}}\n")
    }
}

#[derive(Debug)]
///Stmt is BaseAST
pub struct Stmt {
    pub num: i32,
}
/// 为 Stmt 结构体实现 Display trait
impl fmt::Display for Stmt {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(_f, "    ret {}\n", self.num)
    }
}
