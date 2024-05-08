//! 根据内存形式 Koopa IR 生成汇编
use std::{fs::File, io::Write};

use koopa::ir::*;
pub trait GenerateAsm {
    fn generate(&self, output: &mut File);
}

impl GenerateAsm for Program {
    fn generate(&self, output: &mut File) {
        writeln!(output, "  .text").unwrap();
        // 遍历函数列表
        for &func in self.func_layout() {
            self.func(func).generate(output);
        }
    }
}

impl GenerateAsm for koopa::ir::FunctionData {
    fn generate(&self, output: &mut File) {
        // 遍历基本块列表
        writeln!(output, "  .globl {}", &self.name()[1..]).unwrap();
        writeln!(output, "{}:", &self.name()[1..]).unwrap();
        for (&bb, node) in self.layout().bbs() {
            // 一些必要的处理
            //...
            // 遍历指令列表
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                // 访问指令
                match value_data.kind() {
                    ValueKind::Return(ret_inst) => {
                        // 处理 ret 指令
                        match ret_inst.value() {
                            Some(ret_val) => {
                                let ret_data = self.dfg().value(ret_val);
                                match ret_data.kind() {
                                    ValueKind::Integer(val) => {
                                        writeln!(output, "  li a0, {}", val.value()).unwrap();
                                        writeln!(output, "  ret").unwrap();
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    // 其他种类暂时遇不到
                    _ => unreachable!(),
                }
            }
        }
    }
}
