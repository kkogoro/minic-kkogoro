//!一些处理数组的函数

use crate::ast::*;
use crate::calc_exp::Eval;
use crate::ds_for_ir::GenerateIrInfo;
use crate::symbol_table::ArrayInfoBase;
use crate::symbol_table::SymbolInfo::Array;
use std::{fs::File, io::Write};

///数组定义处理接口
pub trait GenDefDim {
    ///生成数组维度声明
    fn gen_def_dim(&self, output: &mut File, info: &mut GenerateIrInfo) -> Vec<i32>;
}

impl GenDefDim for ConstDef {
    fn gen_def_dim(&self, output: &mut File, info: &mut GenerateIrInfo) -> Vec<i32> {
        //遍历dims，计算数组维度大小
        let mut real_dims: Vec<i32> = vec![];
        for dim in &self.dims {
            real_dims.push(dim.eval(info).expect("数组维度中出现非常量表达式"));
        }
        //插入符号表，标明是数组
        info.insert_symbol(
            self.ident.clone(),
            Array(ArrayInfoBase {
                dims: real_dims.clone(), //borrow
            }),
        );
        //生成维度声明
        match info.is_global_symbol(&self.ident) {
            true => write!(output, "global @{} = alloc ", info.get_name(&self.ident)).unwrap(),
            false => write!(output, "  @{} = alloc ", info.get_name(&self.ident)).unwrap(),
        }
        let left = "[".to_string().repeat(real_dims.len());
        write!(output, "{}i32", left).unwrap();
        for dim in real_dims.iter().rev() {
            write!(output, ", {}]", dim).unwrap();
        }
        real_dims
    }
}

impl GenDefDim for VarDef {
    fn gen_def_dim(&self, output: &mut File, info: &mut GenerateIrInfo) -> Vec<i32> {
        //遍历dims，计算数组维度大小
        let mut real_dims: Vec<i32> = vec![];
        for dim in &self.dims {
            real_dims.push(dim.eval(info).expect("数组维度中出现非常量表达式"));
        }
        //插入符号表，标明是数组
        info.insert_symbol(
            self.ident.clone(),
            Array(ArrayInfoBase {
                dims: real_dims.clone(), //borrow
            }),
        );
        //生成维度声明
        match info.is_global_symbol(&self.ident) {
            true => write!(output, "global @{} = alloc ", info.get_name(&self.ident)).unwrap(),
            false => write!(output, "  @{} = alloc ", info.get_name(&self.ident)).unwrap(),
        }
        let left = "[".to_string().repeat(real_dims.len());
        write!(output, "{}i32", left).unwrap();
        for dim in real_dims.iter().rev() {
            write!(output, ", {}]", dim).unwrap();
        }
        real_dims
    }
}

///对全局数组的初始化
pub trait GlobalArrayInit {
    fn gen_array_init(
        &self,
        output: &mut File,
        info: &mut GenerateIrInfo,
        dims: &[i32],
        result: &mut Vec<i32>,
    );
}

impl GlobalArrayInit for ConstInitVal {
    fn gen_array_init(
        &self,
        output: &mut File,
        info: &mut GenerateIrInfo,
        dims: &[i32],
        result: &mut Vec<i32>,
    ) {
        match self {
            ConstInitVal::ConstExp(exp) => {
                let val = exp.eval(info).expect("数组初始化中出现非常量表达式");
                result.push(val);
            }
            ConstInitVal::ConstInitValS(vals) => {
                let pre_filled = result.len();
                for val in vals {
                    let now_filled = result.len();
                    let mut align_dim = dims.len(); //对齐到哪个维度 TODO:check

                    let mut align_size: usize = 1; //对齐到的维度对应大小

                    for it in dims.iter().rev() {
                        align_size *= *it as usize;
                        if now_filled % align_size == 0 {
                            align_dim -= 1;
                        } else {
                            break;
                        }
                    }

                    if align_dim == 0 {
                        //初始值，啥都没填充，对齐到第一维
                        align_dim = 1;
                    }

                    /*  if align_dim < dims.len() {
                        //每个花括号还要更深入一层，注意到ConstExp实际上还要更多一层
                        align_dim += 1;
                    }*/

                    val.gen_array_init(
                        output,
                        info,
                        &dims[align_dim..dims.len()], //切片的右面是开区间
                        result,
                    );
                }
                let fin_filled = result.len();
                let required_size = dims.iter().fold(1, |acc, x| acc * x) as usize;
                for _ in (fin_filled - pre_filled)..required_size {
                    result.push(0);
                }
            }
        }
    }
}

impl GlobalArrayInit for InitVal {
    fn gen_array_init(
        &self,
        output: &mut File,
        info: &mut GenerateIrInfo,
        dims: &[i32],
        result: &mut Vec<i32>,
    ) {
        match self {
            InitVal::Exp(exp) => {
                let val = exp.eval(info).expect("全局数组初始化中出现非常量表达式");
                result.push(val);
            }
            InitVal::InitValS(vals) => {
                let pre_filled = result.len();
                for val in vals {
                    let now_filled = result.len();
                    let mut align_dim = dims.len(); //对齐到哪个维度 TODO:check

                    let mut align_size: usize = 1; //对齐到的维度对应大小

                    for it in dims.iter().rev() {
                        align_size *= *it as usize;
                        if now_filled % align_size == 0 {
                            align_dim -= 1;
                        } else {
                            break;
                        }
                    }

                    if align_dim == 0 {
                        //初始值，啥都没填充，对齐到第一维
                        align_dim = 1;
                    }

                    /*  if align_dim < dims.len() {
                        //每个花括号还要更深入一层，注意到ConstExp实际上还要更多一层
                        align_dim += 1;
                    }*/

                    val.gen_array_init(
                        output,
                        info,
                        &dims[align_dim..dims.len()], //切片的右面是开区间
                        result,
                    );
                }
                let fin_filled = result.len();
                let required_size = dims.iter().fold(1, |acc, x| acc * x) as usize;
                for _ in (fin_filled - pre_filled)..required_size {
                    result.push(0);
                }
            }
        }
    }
}
