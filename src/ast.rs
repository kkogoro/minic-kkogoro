//! 基础的AST定义

///////////////////////////BaseAST////////////////////////////

#[derive(Debug)]
///CompUnit    ::= {CompItem};
pub struct CompUnit {
    pub item: Vec<CompItem>,
}

#[derive(Debug)]
///CompItem   ::= FuncDef | Decl ;
///TODO
pub enum CompItem {
    FuncDef(FuncDef),
    Decl(Decl),
}

#[derive(Debug)]
///FuncDef     ::= FuncType IDENT "(" [FuncFParams] ")" Block;
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,                 //IDENT
    pub func_fparams: Vec<FuncFParam>, //FuncFParams ::= FuncFParam {"," FuncFParam};
    pub block: Block,
}

#[derive(Debug)]
///FuncFParam ::= BType IDENT ["[" "]" {"[" ConstExp "]"}];
pub struct FuncFParam {
    pub btype: BType,
    pub ident: String,
    pub dims: Option<Vec<ConstExp>>,
}

#[derive(Debug, Copy, Clone)]
///FuncType    ::= "void" | "int";
/// 枚举类型，表示函数的返回值类型
pub enum FuncType {
    Int,
    Void,
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
/// Stmt ::= LVal "=" Exp ";"
///       | [Exp] ";"
///       | Block
///       | "return" [Exp] ";";
///       | "if" "(" Exp ")" Stmt ["else" Stmt]
///       | "while" "(" Exp ")" Stmt
///       | "break" ";"
///       | "continue" ";"
pub enum Stmt {
    Assign(LVal, Exp),
    Exp(Option<Exp>),
    Block(Block),
    If(Exp, Box<Stmt>, Option<Box<Stmt>>),
    RetExp(Option<Exp>),
    While(Exp, Box<Stmt>),
    Break,
    Continue,
}

///////////////////////////Exp////////////////////////////

#[derive(Debug)]
///Exp         ::= LOrExp;
pub enum Exp {
    LOrExp(Box<LOrExp>),
}

#[derive(Debug)]
///UnaryExp    ::= PrimaryExp
///             | UnaryOp UnaryExp;
///             | IDENT "(" [FuncRParams] ")"
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    BinaryOp(UnaryOp, Box<UnaryExp>), //改名
    Call(String, Vec<Exp>),           //FuncRParams ::= Exp {"," Exp};
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
///ConstDef      ::= IDENT {"[" ConstExp "]"} "=" ConstInitVal;
pub struct ConstDef {
    pub ident: String,
    pub dims: Vec<ConstExp>,
    pub const_init_val: ConstInitVal,
}

#[derive(Debug)]
///ConstInitVal  ::= ConstExp | "{" [ConstInitVal {"," ConstInitVal}] "}";
pub enum ConstInitVal {
    ConstExp(ConstExp),
    ConstInitValS(Vec<ConstInitVal>),
}

#[derive(Debug)]
///ConstExp      ::= Exp;
pub enum ConstExp {
    Exp(Exp),
}

#[derive(Debug)]
///LVal          ::= IDENT {"[" Exp "]"};
/// IDENT对应Ident
pub struct LVal {
    pub ident: String,
    pub dims: Vec<Exp>,
}

#[derive(Debug)]
///VarDecl       ::= BType VarDef {"," VarDef} ";";
pub enum VarDecl {
    VarDeclS(BType, Vec<VarDef>),
}

#[derive(Debug)]
///VarDef        ::= IDENT {"[" ConstExp "]"}
///                | IDENT {"[" ConstExp "]"} "=" InitVal;
pub struct VarDef {
    pub ident: String,
    pub dims: Vec<ConstExp>,
    pub init_val: Option<InitVal>,
}

#[derive(Debug)]
///InitVal       ::= Exp | "{" [InitVal {"," InitVal}] "}";
pub enum InitVal {
    Exp(Exp),
    InitValS(Vec<InitVal>),
}
