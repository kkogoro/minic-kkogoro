//! 基础的AST定义

use koopa::ir::values::Binary;

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
///Exp         ::= LOrExp;
pub enum Exp {
    LOrExp(Box<LOrExp>),
}

#[derive(Debug)]
///UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    BinaryOp(UnaryOp, Box<UnaryExp>), //改名
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

#[derive(Debug)]
///BinaryAddOP ::= "+" | "-" ;
pub enum BinaryAddOp {
    Add,
    Sub,
}

#[derive(Debug)]
///BinaryMulOp ::= "*" | "/" | "%" ;
pub enum BinaryMulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
///AddExp      ::= MulExp | AddExp ("+" | "-") MulExp;
pub enum AddExp {
    MulExp(Box<MulExp>),
    BinaryExp(Box<AddExp>, BinaryAddOp, Box<MulExp>),
}

#[derive(Debug)]
///MulExp      ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
pub enum MulExp {
    UnaryExp(Box<UnaryExp>),
    BinaryExp(Box<MulExp>, BinaryMulOp, Box<UnaryExp>),
}

#[derive(Debug)]
///BinaryRelOp ::= "<" | ">" | "<=" | ">=" ;
pub enum BinaryRelOp {
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug)]
///RelExp      ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
pub enum RelExp {
    AddExp(Box<AddExp>),
    BinaryExp(Box<RelExp>, BinaryRelOp, Box<AddExp>),
}

#[derive(Debug)]
///BinaryEqOp  ::= "==" | "!=" ;
pub enum BinaryEqOp {
    Eq,
    Ne,
}

#[derive(Debug)]
///EqExp       ::= RelExp | EqExp ("==" | "!=") RelExp;
pub enum EqExp {
    RelExp(Box<RelExp>),
    BinaryExp(Box<EqExp>, BinaryEqOp, Box<RelExp>),
}

#[derive(Debug)]
///LAndExp     ::= EqExp | LAndExp "&&" EqExp;
pub enum LAndExp {
    EqExp(Box<EqExp>),
    BinaryExp(Box<LAndExp>, Box<EqExp>),
}

#[derive(Debug)]
///LOrExp      ::= LAndExp | LOrExp "||" LAndExp;
pub enum LOrExp {
    LAndExp(Box<LAndExp>),
    BinaryExp(Box<LOrExp>, Box<LAndExp>),
}
