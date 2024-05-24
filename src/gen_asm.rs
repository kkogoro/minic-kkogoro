//! 根据内存形式 Koopa IR 生成汇编
use std::{fs::File, io::Write};

use crate::ds_for_asm::GenerateAsmInfo;
use koopa::ir::*;
pub trait GenerateAsm {
    type GenerateResult;
    fn generate(&self, output: &mut File) -> Self::GenerateResult;
}

fn get_reg(
    output: &mut File,
    func_data: &koopa::ir::FunctionData,
    func_info: &mut GenerateAsmInfo,
    value: Value,
) -> String {
    let val_data = func_data.dfg().value(value);
    match val_data.kind() {
        ValueKind::Integer(val) => {
            if val.value() == 0 {
                return "x0".to_string();
            }
            let reg = func_info.get_reg(value).expect("寄存器不足");
            writeln!(output, "  li {}, {}", reg, val.value()).unwrap();
            reg
        }
        _ => func_info.get_reg(value).expect("寄存器不足"), //expr
    }
}

/// 为Program实现GenerateAsm trait
impl GenerateAsm for Program {
    type GenerateResult = ();
    fn generate(&self, output: &mut File) {
        writeln!(output, "  .text").unwrap();
        // 遍历函数列表
        for &func in self.func_layout() {
            self.func(func).generate(output);
        }
    }
}

/// 为FunctionData实现GenerateAsm trait
impl GenerateAsm for koopa::ir::FunctionData {
    type GenerateResult = ();
    fn generate(&self, output: &mut File) {
        //跳过声明
        if self.layout().entry_bb().is_none() {
            return;
        }

        let mut func_info = GenerateAsmInfo::new();

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
                                    ValueKind::Binary(bin_inst) => {
                                        let reg_ret =
                                            get_reg(output, self, &mut func_info, ret_val);
                                        writeln!(output, "  mv a0, {}", reg_ret).unwrap();
                                        writeln!(output, "  ret").unwrap();
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    ValueKind::Binary(bin_inst) => {
                        // 处理二元运算
                        let lhs = bin_inst.lhs();
                        let rhs = bin_inst.rhs();

                        let reg_l = get_reg(output, self, &mut func_info, lhs);

                        let reg_r = get_reg(output, self, &mut func_info, rhs);

                        let reg_ret = get_reg(output, self, &mut func_info, inst); //指令ID对应结果

                        match bin_inst.op() {
                            BinaryOp::Add => {
                                writeln!(output, "  add {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Sub => {
                                writeln!(output, "  sub {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Mul => {
                                writeln!(output, "  mul {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Div => {
                                writeln!(output, "  div {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Mod => {
                                writeln!(output, "  rem {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::And => {
                                writeln!(output, "  and {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Or => {
                                writeln!(output, "  or {}, {}, {}", reg_ret, reg_l, reg_r).unwrap();
                            }
                            BinaryOp::Xor => {
                                writeln!(output, "  xor {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Shl => {
                                writeln!(output, "  sll {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Shr => {
                                writeln!(output, "  srl {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Sar => {
                                writeln!(output, "  sra {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                    // 其他种类暂时遇不到
                    _ => {}
                }
            }
        }
    }
}
