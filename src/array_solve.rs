//!一些处理数组的函数

use crate::ast::*;
use crate::calc_exp::Eval;
use crate::ds_for_ir::GenerateIrInfo;
use crate::symbol_table::ArrayInfoBase;
use crate::symbol_table::SymbolInfo::Array;
use std::{fs::File, io::Write};

pub trait GenDefDim {
    fn gen_def_dim(&self, output: &mut File, info: &mut GenerateIrInfo);
}

impl GenDefDim for ConstDef {
    fn gen_def_dim(&self, output: &mut File, info: &mut GenerateIrInfo) {
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
    }
}

impl GenDefDim for VarDef {
    fn gen_def_dim(&self, output: &mut File, info: &mut GenerateIrInfo) {
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
    }
}
