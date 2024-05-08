///!实现生成Koopa IR
use std::{fs::File, io::Write};

use crate::ast::*;
static mut NOW_INDENT: i32 = 0;

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
                    writeln!(output, "  ret %{}", NOW_INDENT).unwrap();
                }
            }
        }
    }
}

///为Exp实现GenerateIR trait
impl GenerateIR for Exp {
    fn generate(&self, output: &mut File) {
        match self {
            Exp::AddExp(add_exp) => add_exp.generate(output),
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
                match op {
                    UnaryOp::Neg => unsafe {
                        NOW_INDENT += 1;
                        writeln!(output, "  %{} = sub 0, %{}", NOW_INDENT, NOW_INDENT - 1).unwrap();
                    },
                    UnaryOp::Pos => {}
                    UnaryOp::Not => unsafe {
                        NOW_INDENT += 1;
                        writeln!(output, "  %{} = eq 0, %{}", NOW_INDENT, NOW_INDENT - 1).unwrap();
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
                //write!(output, "(").unwrap();
                exp.generate(output);
                //write!(output, ")").unwrap();
            }
            PrimaryExp::Number(num) => unsafe {
                NOW_INDENT += 1;
                //这里以后回来改
                writeln!(output, "  %{} = add {}, 0", NOW_INDENT, num).unwrap();
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
                add_exp.generate(output);
                unsafe {
                    NOW_INDENT += 1;
                    write!(output, "  %{} = ", NOW_INDENT).unwrap();
                    match op {
                        BinaryAddOp::Add => write!(output, "add").unwrap(),
                        BinaryAddOp::Sub => write!(output, "sub").unwrap(),
                        _ => panic!("Wrong Op in AddExp"),
                    }
                    writeln!(output, " %{}, %{}", NOW_INDENT - 1, NOW_INDENT - 2).unwrap();
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
                mul_exp.generate(output);
                unsafe {
                    NOW_INDENT += 1;
                    write!(output, " %{} = ", NOW_INDENT).unwrap();
                    match op {
                        BinaryMulOp::Mul => write!(output, "mul").unwrap(),
                        BinaryMulOp::Div => write!(output, "div").unwrap(),
                        BinaryMulOp::Mod => write!(output, "mod").unwrap(),
                        _ => panic!("Wrong Op in MulExp"),
                    }
                    writeln!(output, " %{}, %{}", NOW_INDENT - 1, NOW_INDENT - 2).unwrap();
                }
            }
        }
    }
}
