//! 基础的AST定义

///////////////////////////BaseAST////////////////////////////

#[derive(Debug)]
///CompUnit is BaseAST
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
///FuncDef is BaseAST
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String, //IDENT
    pub block: Block,
}

#[derive(Debug)]
///FuncType is BaseAST
/// 枚举类型，表示函数的返回值类型
pub enum FuncType {
    Int,
}

#[derive(Debug)]
///Block         ::= "{" {BlockItem} "}";
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
///BlockItem     ::= Decl | Stmt;
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug)]
///Stmt          ::= LVal "=" Exp ";"
///                | "return" Exp ";";
pub enum Stmt {
    RetExp(Exp),
    Assign(LVal, Exp),
}

///////////////////////////Exp////////////////////////////

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
///PrimaryExp    ::= "(" Exp ")" | LVal | Number;
pub enum PrimaryExp {
    Bexp(Box<Exp>),
    LVal(LVal),
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

#[derive(Debug)]
///Decl          ::= ConstDecl | VarDecl;
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl),
}

#[derive(Debug)]
///ConstDecl     ::= "const" BType ConstDef {"," ConstDef} ";";
pub enum ConstDecl {
    ConstDeclS(BType, Vec<ConstDef>),
}

#[derive(Debug)]
///BType         ::= "int";
pub enum BType {
    Int,
}

#[derive(Debug)]
///ConstDef      ::= IDENT "=" ConstInitVal;
pub struct ConstDef {
    pub ident: String,
    pub const_init_val: ConstInitVal,
}

#[derive(Debug)]
///ConstInitVal  ::= ConstExp;
pub enum ConstInitVal {
    ConstExp(ConstExp),
}

#[derive(Debug)]
///ConstExp      ::= Exp;
pub enum ConstExp {
    Exp(Exp),
}

#[derive(Debug)]
///LVal          ::= IDENT;
/// IDENT对应Ident
pub struct LVal {
    pub ident: String,
}

#[derive(Debug)]
///VarDecl       ::= BType VarDef {"," VarDef} ";";
pub enum VarDecl {
    VarDeclS(BType, Vec<VarDef>),
}

#[derive(Debug)]
///VarDef        ::= IDENT | IDENT "=" InitVal;
pub enum VarDef {
    NoInit(String),
    Init(String, InitVal),
}

#[derive(Debug)]
///InitVal       ::= Exp;
pub enum InitVal {
    Exp(Exp),
}
