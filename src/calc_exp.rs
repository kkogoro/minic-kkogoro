///!实现生成表达式静态求值
use crate::ast::*;
use crate::ds_for_ir::GenerateIrInfo;
use crate::symbol_table::SymbolType;

pub trait Eval {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32>;
}

///为ConstInitVal实现Eval trait
impl Eval for ConstInitVal {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            ConstInitVal::ConstExp(const_exp) => const_exp.eval(info),
        }
    }
}

///为ConstExp实现Eval trait
impl Eval for ConstExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            ConstExp::Exp(exp) => exp.eval(info),
        }
    }
}

///为Exp实现Eval trait
impl Eval for Exp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            Exp::LOrExp(exp) => exp.eval(info),
        }
    }
}

///为LOrExp实现Eval trait
impl Eval for LOrExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            LOrExp::LAndExp(exp) => exp.eval(info),
            LOrExp::BinaryExp(exp1, exp2) => match (exp1.eval(info), exp2.eval(info)) {
                (Some(v1), Some(v2)) => {
                    if v1 != 0 || v2 != 0 {
                        Some(1)
                    } else {
                        Some(0)
                    }
                }
                _ => None,
            },
        }
    }
}

///为LAndExp实现Eval trait
impl Eval for LAndExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            LAndExp::EqExp(exp) => exp.eval(info),
            LAndExp::BinaryExp(exp1, exp2) => match (exp1.eval(info), exp2.eval(info)) {
                (Some(v1), Some(v2)) => {
                    if v1 == 0 || v2 == 0 {
                        Some(0)
                    } else {
                        Some(1)
                    }
                }
                _ => None,
            },
        }
    }
}

///为EqExp实现Eval trait
impl Eval for EqExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            EqExp::RelExp(exp) => exp.eval(info),
            EqExp::BinaryExp(exp1, op, exp2) => match (exp1.eval(info), exp2.eval(info)) {
                (Some(v1), Some(v2)) => match op {
                    BinaryEqOp::Eq => {
                        if v1 == v2 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                    BinaryEqOp::Ne => {
                        if v1 != v2 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                },
                _ => None,
            },
        }
    }
}

///为RelExp实现Eval trait
impl Eval for RelExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            RelExp::AddExp(exp) => exp.eval(info),
            RelExp::BinaryExp(exp1, op, exp2) => match (exp1.eval(info), exp2.eval(info)) {
                (Some(v1), Some(v2)) => match op {
                    BinaryRelOp::Lt => {
                        if v1 < v2 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                    BinaryRelOp::Gt => {
                        if v1 > v2 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                    BinaryRelOp::Le => {
                        if v1 <= v2 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                    BinaryRelOp::Ge => {
                        if v1 >= v2 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                },
                _ => None,
            },
        }
    }
}

///为AddExp实现Eval trait
impl Eval for AddExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            AddExp::MulExp(exp) => exp.eval(info),
            AddExp::BinaryExp(exp1, op, exp2) => match (exp1.eval(info), exp2.eval(info)) {
                (Some(v1), Some(v2)) => match op {
                    BinaryAddOp::Add => Some(v1 + v2),
                    BinaryAddOp::Sub => Some(v1 - v2),
                },
                _ => None,
            },
        }
    }
}

///为MulExp实现Eval trait
impl Eval for MulExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            MulExp::UnaryExp(exp) => exp.eval(info),
            MulExp::BinaryExp(exp1, op, exp2) => match (exp1.eval(info), exp2.eval(info)) {
                (Some(v1), Some(v2)) => match op {
                    BinaryMulOp::Mul => Some(v1 * v2),
                    BinaryMulOp::Div => Some(v1 / v2),
                    BinaryMulOp::Mod => Some(v1 % v2),
                },
                _ => None,
            },
        }
    }
}

///为UnaryExp实现Eval trait
impl Eval for UnaryExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            UnaryExp::PrimaryExp(exp) => exp.eval(info),
            UnaryExp::BinaryOp(op, exp) => match exp.eval(info) {
                None => None,
                Some(v) => match op {
                    UnaryOp::Neg => Some(-v),
                    UnaryOp::Pos => Some(v),
                    UnaryOp::Not => {
                        if v == 0 {
                            Some(1)
                        } else {
                            Some(0)
                        }
                    }
                },
            },
        }
    }
}

///为PrimaryExp实现Eval trait
impl Eval for PrimaryExp {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        match self {
            PrimaryExp::Bexp(exp) => exp.eval(info),
            PrimaryExp::Number(num) => Some(*num),
            PrimaryExp::LVal(lval) => lval.eval(info),
        }
    }
}

///为LVal实现Eval trait
///注意到LVal只有为常量时才可调用Eval
///可在更上一层调用中就得知LVal是否为常量
impl Eval for LVal {
    fn eval(&self, info: &mut GenerateIrInfo) -> Option<i32> {
        let val = info.search_symbol(&self.ident).expect("No Symbol Found!");
        match val.content {
            SymbolType::Const(v) => Some(v),
            SymbolType::Var(_) => None,
        }
    }
}
