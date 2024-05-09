///!实现生成Koopa IR
use std::{fs::File, io::Write};

use crate::ast::*;
static mut NOW_ID: i32 = 0;

pub trait GenerateIR {
    fn generate(&self, output: &mut File);
}

///为CompUnit实现 GenerateIR trait
impl GenerateIR for CompUnit {
    fn generate(&self, output: &mut File) {
        self.func_def.generate(output);
    }
}

///为FuncDef实现GenerateIR trait
impl GenerateIR for FuncDef {
    fn generate(&self, output: &mut File) {
        write!(output, "fun @{}", self.ident).unwrap();
        write!(output, "(): ").unwrap();
        self.func_type.generate(output);
        write!(output, " ").unwrap();
        self.block.generate(output);
    }
}

///为FuncType实现GenerateIR trait
impl GenerateIR for FuncType {
    fn generate(&self, output: &mut File) {
        match self {
            FuncType::Int => write!(output, "i32").unwrap(),
        }
    }
}

///为Block实现GenerateIR trait
impl GenerateIR for Block {
    fn generate(&self, output: &mut File) {
        write!(output, "{{\n").unwrap();
        write!(output, "%entry:\n").unwrap();
        self.stmt.generate(output);
        write!(output, "}}\n").unwrap();
    }
}

///为Stmt实现GenerateIR trait
impl GenerateIR for Stmt {
    fn generate(&self, output: &mut File) {
        match self {
            Stmt::RetExp(exp) => {
                exp.generate(output);
                unsafe {
                    writeln!(output, "  ret %{}", NOW_ID).unwrap();
                }
            }
        }
    }
}

///为Exp实现GenerateIR trait
impl GenerateIR for Exp {
    fn generate(&self, output: &mut File) {
        match self {
            Exp::LOrExp(lor_exp) => lor_exp.generate(output),
        }
    }
}

///为UnaryExp实现GenerateIR trait
impl GenerateIR for UnaryExp {
    fn generate(&self, output: &mut File) {
        match self {
            UnaryExp::PrimaryExp(primary_exp) => primary_exp.generate(output),
            UnaryExp::BinaryOp(op, exp) => {
                exp.generate(output);
                let exp_id = unsafe { NOW_ID };

                match op {
                    UnaryOp::Neg => unsafe {
                        NOW_ID += 1;
                        writeln!(output, "  %{} = sub 0, %{}", NOW_ID, exp_id).unwrap();
                    },
                    UnaryOp::Pos => {}
                    UnaryOp::Not => unsafe {
                        NOW_ID += 1;
                        writeln!(output, "  %{} = eq 0, %{}", NOW_ID, exp_id).unwrap();
                    },
                }
            }
        }
    }
}

///为PrimaryExp实现GenerateIR trait
impl GenerateIR for PrimaryExp {
    fn generate(&self, output: &mut File) {
        match self {
            PrimaryExp::Bexp(exp) => {
                exp.generate(output);
            }
            PrimaryExp::Number(num) => unsafe {
                NOW_ID += 1;
                //这里以后回来改
                writeln!(output, "  %{} = add {}, 0", NOW_ID, num).unwrap();
            },
        }
    }
}

///为AddExp实现GenerateIR trait
impl GenerateIR for AddExp {
    fn generate(&self, output: &mut File) {
        match self {
            AddExp::MulExp(mul_exp) => mul_exp.generate(output),
            AddExp::BinaryExp(add_exp, op, mul_exp) => {
                mul_exp.generate(output);
                let mul_id = unsafe { NOW_ID };
                add_exp.generate(output);
                let add_id = unsafe { NOW_ID };
                unsafe {
                    NOW_ID += 1;
                    write!(output, "  %{} = ", NOW_ID).unwrap();
                    match op {
                        BinaryAddOp::Add => write!(output, "add").unwrap(),
                        BinaryAddOp::Sub => write!(output, "sub").unwrap(),
                        _ => panic!("Wrong Op in AddExp"),
                    }
                    writeln!(output, " %{}, %{}", add_id, mul_id).unwrap();
                }
            }
        }
    }
}

///为MulExp实现GenerateIR trait
impl GenerateIR for MulExp {
    fn generate(&self, output: &mut File) {
        match self {
            MulExp::UnaryExp(unary_exp) => unary_exp.generate(output),
            MulExp::BinaryExp(mul_exp, op, unary_exp) => {
                unary_exp.generate(output);
                let unary_id = unsafe { NOW_ID };
                mul_exp.generate(output);
                let mul_id = unsafe { NOW_ID };
                unsafe {
                    NOW_ID += 1;
                    write!(output, "  %{} = ", NOW_ID).unwrap();
                    match op {
                        BinaryMulOp::Mul => write!(output, "mul").unwrap(),
                        BinaryMulOp::Div => write!(output, "div").unwrap(),
                        BinaryMulOp::Mod => write!(output, "mod").unwrap(),
                        _ => panic!("Wrong Op in MulExp"),
                    }
                    writeln!(output, " %{}, %{}", mul_id, unary_id).unwrap();
                }
            }
        }
    }
}

///为RelExp实现GenerateIR trait
impl GenerateIR for RelExp {
    fn generate(&self, output: &mut File) {
        match self {
            RelExp::AddExp(add_exp) => add_exp.generate(output),
            RelExp::BinaryExp(rel_exp, op, add_exp) => {
                add_exp.generate(output);
                let add_id = unsafe { NOW_ID };
                rel_exp.generate(output);
                let rel_id = unsafe { NOW_ID };
                unsafe {
                    NOW_ID += 1;
                    write!(output, "  %{} = ", NOW_ID).unwrap();
                    match op {
                        BinaryRelOp::Lt => write!(output, "lt").unwrap(),
                        BinaryRelOp::Gt => write!(output, "gt").unwrap(),
                        BinaryRelOp::Le => write!(output, "le").unwrap(),
                        BinaryRelOp::Ge => write!(output, "ge").unwrap(),
                    }
                    writeln!(output, " %{}, %{}", rel_id, add_id).unwrap();
                }
            }
        }
    }
}

///为EqExp实现GenerateIR trait
impl GenerateIR for EqExp {
    fn generate(&self, output: &mut File) {
        match self {
            EqExp::RelExp(rel_exp) => rel_exp.generate(output),
            EqExp::BinaryExp(eq_exp, op, rel_exp) => {
                rel_exp.generate(output);
                let rel_id = unsafe { NOW_ID };
                eq_exp.generate(output);
                let eq_id = unsafe { NOW_ID };
                unsafe {
                    NOW_ID += 1;
                    write!(output, "  %{} = ", NOW_ID).unwrap();
                    match op {
                        BinaryEqOp::Eq => write!(output, "eq").unwrap(),
                        BinaryEqOp::Ne => write!(output, "ne").unwrap(),
                    }
                    writeln!(output, " %{}, %{}", eq_id, rel_id).unwrap();
                }
            }
        }
    }
}

///为LAndExp实现GenerateIR trait
///注意应该是实现逻辑and，Koopa IR中的是按位and
impl GenerateIR for LAndExp {
    fn generate(&self, output: &mut File) {
        match self {
            LAndExp::EqExp(eq_exp) => eq_exp.generate(output),
            LAndExp::BinaryExp(land_exp, eq_exp) => {
                eq_exp.generate(output);
                let eq_id = unsafe { NOW_ID };
                land_exp.generate(output);
                let land_id = unsafe { NOW_ID };
                unsafe {
                    //eq != 0
                    NOW_ID += 1;
                    let eq_not_0 = NOW_ID;
                    writeln!(output, "  %{} = ne 0, %{}", eq_not_0, eq_id).unwrap();

                    //land != 0
                    NOW_ID += 1;
                    let land_not_0 = NOW_ID;
                    writeln!(output, "  %{} = ne 0, %{}", land_not_0, land_id).unwrap();

                    //(eq != 0) & (land != 0)
                    NOW_ID += 1;
                    writeln!(output, "  %{} = and %{}, %{}", NOW_ID, land_not_0, eq_not_0).unwrap();
                }
            }
        }
    }
}

///为LOrExp实现GenerateIR trait
impl GenerateIR for LOrExp {
    fn generate(&self, output: &mut File) {
        match self {
            LOrExp::LAndExp(land_exp) => land_exp.generate(output),
            LOrExp::BinaryExp(lor_exp, land_exp) => {
                land_exp.generate(output);
                let land_id = unsafe { NOW_ID };
                lor_exp.generate(output);
                let lor_id = unsafe { NOW_ID };
                unsafe {
                    //land != 0
                    NOW_ID += 1;
                    let land_not_0 = NOW_ID;
                    writeln!(output, "  %{} = ne 0, %{}", land_not_0, land_id).unwrap();

                    //lor != 0
                    NOW_ID += 1;
                    let lor_not_0 = NOW_ID;
                    writeln!(output, "  %{} = ne 0, %{}", lor_not_0, lor_id).unwrap();

                    //(lor != 0) | (land != 0)
                    NOW_ID += 1;
                    writeln!(output, "  %{} = or %{}, %{}", NOW_ID, lor_not_0, land_not_0).unwrap();
                }
            }
        }
    }
}
