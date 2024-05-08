//! 基础的AST定义

#[derive(Debug)]
///CompUnit is BaseAST
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
///FuncDef is BaseAST
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

#[derive(Debug)]
///FuncType is BaseAST
/// 枚举类型，表示函数的返回值类型
pub enum FuncType {
    Int,
}

#[derive(Debug)]
///Block is BaseAST
pub struct Block {
    pub stmt: Stmt,
}

#[derive(Debug)]
///Stmt        ::= "return" Exp ";";
pub enum Stmt {
    RetExp(Exp),
}

#[derive(Debug)]
///Exp         ::= UnaryExp;
pub enum Exp {
    UnaryExp(UnaryExp),
}

#[derive(Debug)]
///UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    BinaryOp(UnaryOp, Box<UnaryExp>),
}

#[derive(Debug)]
///PrimaryExp  ::= "(" Exp ")" | Number;
pub enum PrimaryExp {
    Bexp(Box<Exp>),
    Number(i32),
}

#[derive(Debug)]
///UnaryOp     ::= "+" | "-" | "!";
pub enum UnaryOp {
    Neg,
    Pos,
    Not,
}
