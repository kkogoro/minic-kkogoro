//! 根据内存形式 Koopa IR 生成汇编
use std::{fs::File, io::Write};

use crate::ds_for_asm::check_i12;
use crate::ds_for_asm::GenerateAsmInfo;
use crate::ds_for_asm::UserKind;
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
            let reg = func_info.get_reg_i32(output, val.value());
            reg
        }
        _ => func_info.get_reg(output, value), //expr
    }
}

///释放寄存器，TODO:先都释放掉，后面加spill再把用到这个的都删掉
fn free_reg(func_data: &koopa::ir::FunctionData, func_info: &mut GenerateAsmInfo, value: Value) {
    let val_data = func_data.dfg().value(value);
    match val_data.kind() {
        ValueKind::Integer(val) => {
            if val.value() == 0 {
                //0不需要释放，之前用的x0
                return;
            }
            func_info.free_reg(UserKind::Tmpi32(val.value()));
        }
        _ => {
            func_info.free_reg(UserKind::Val(value));
        }
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

        writeln!(output, "  .globl {}", &self.name()[1..]).unwrap();
        writeln!(output, "{}:", &self.name()[1..]).unwrap();

        let mut func_info = GenerateAsmInfo::new();

        //计算栈大小
        for (&bb, node) in self.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                let value_type = value_data.ty();
                if value_type.is_unit() {
                    //没有返回值，不需要分配栈空间
                    continue;
                }
                func_info.stack_size += value_type.size() as i32;
            }
        }

        //对齐16
        func_info.stack_size = (func_info.stack_size + 15) & !15; //check?

        //移动栈指针
        func_info.set_sp(output);

        //为每个元素分配栈偏移量
        let mut now_stack_offset = 0;

        // 遍历基本块列表
        for (&bb, node) in self.layout().bbs() {
            // 一些必要的处理
            let bb_data = self.dfg().bb(bb);
            let block_name = bb_data.name().clone().unwrap();
            if block_name != "%entry".to_owned() {
                writeln!(output, "{}:", &block_name[1..]).unwrap();
            }

            // 遍历指令列表
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                // 访问指令
                match value_data.kind() {
                    ValueKind::Branch(br_inst) => {
                        let cond = br_inst.cond();
                        let true_bb_name = self.dfg().bb(br_inst.true_bb()).name().clone().unwrap();
                        let false_bb_name =
                            self.dfg().bb(br_inst.false_bb()).name().clone().unwrap();
                        let reg_cond = get_reg(output, self, &mut func_info, cond);
                        writeln!(output, "  bnez {}, {}", reg_cond, &true_bb_name[1..]).unwrap();
                        writeln!(output, "  j {}", &false_bb_name[1..]).unwrap();
                        free_reg(self, &mut func_info, cond);
                    }
                    ValueKind::Jump(jump_inst) => {
                        // 处理 jump 指令
                        let target_bb = jump_inst.target();
                        let target_data = self.dfg().bb(target_bb);
                        let target_name = target_data.name().clone().unwrap();
                        writeln!(output, "  j {}", &target_name[1..]).unwrap();
                    }
                    ValueKind::Return(ret_inst) => {
                        // 处理 ret 指令
                        match ret_inst.value() {
                            Some(ret_val) => {
                                let ret_data = self.dfg().value(ret_val);

                                match ret_data.kind() {
                                    ValueKind::Integer(val) => {
                                        writeln!(output, "  li a0, {}", val.value()).unwrap();
                                    }
                                    _ => {
                                        //其他情况**都**直接从内存读到寄存器？
                                        let reg_ret =
                                            get_reg(output, self, &mut func_info, ret_val);
                                        writeln!(output, "  mv a0, {}", reg_ret).unwrap();

                                        free_reg(self, &mut func_info, ret_val);
                                    }
                                }
                            }
                            None => {}
                        }

                        func_info.reset_sp(output);
                        writeln!(output, "  ret").unwrap();
                    }
                    ValueKind::Alloc(_) => {
                        // 处理 alloc 指令

                        func_info.alloc(output, inst, now_stack_offset);
                        now_stack_offset += value_data.ty().size() as i32;
                    }
                    ValueKind::Load(load_inst) => {
                        // 处理 load 指令
                        let addr = load_inst.src();
                        let offset = *func_info.name_to_offset.get(&addr).unwrap();
                        let reg_ret = get_reg(output, self, &mut func_info, inst);

                        if check_i12(offset) {
                            writeln!(output, "  lw {}, {}(sp)", reg_ret, offset).unwrap();
                        } else {
                            let reg_addr = func_info.get_reg_i32(output, offset);
                            writeln!(output, "  add {}, sp, {}", reg_addr, reg_addr).unwrap();
                            writeln!(output, "  lw {}, 0({})", reg_ret, reg_addr).unwrap();
                            func_info.free_reg(UserKind::Tmpi32(offset));
                        }

                        func_info.new_var(output, inst, now_stack_offset);
                        now_stack_offset += value_data.ty().size() as i32;
                        free_reg(self, &mut func_info, inst);
                    }
                    ValueKind::Store(store_inst) => {
                        // 处理 store 指令
                        let addr = store_inst.dest();
                        let offset = *func_info.name_to_offset.get(&addr).unwrap();
                        let value = store_inst.value();
                        let reg_val = get_reg(output, self, &mut func_info, value);

                        if (check_i12(offset)) {
                            writeln!(output, "  sw {}, {}(sp)", reg_val, offset).unwrap();
                        } else {
                            let reg_addr = func_info.get_reg_i32(output, offset);
                            writeln!(output, "  add {}, sp, {}", reg_addr, reg_addr).unwrap();
                            writeln!(output, "  sw {}, 0({})", reg_val, reg_addr).unwrap();
                            func_info.free_reg(UserKind::Tmpi32(offset));
                        }

                        free_reg(self, &mut func_info, value);
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
                            BinaryOp::Eq => {
                                writeln!(output, "  xor {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                                writeln!(output, "  seqz {}, {}", reg_ret, reg_ret).unwrap();
                            }
                            BinaryOp::NotEq => {
                                writeln!(output, "  xor {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                                writeln!(output, "  snez {}, {}", reg_ret, reg_ret).unwrap();
                            }
                            BinaryOp::Lt => {
                                writeln!(output, "  slt {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Gt => {
                                writeln!(output, "  sgt {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                            }
                            BinaryOp::Le => {
                                writeln!(output, "  sgt {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                                writeln!(output, "  seqz {}, {}", reg_ret, reg_ret).unwrap();
                            }
                            BinaryOp::Ge => {
                                writeln!(output, "  slt {}, {}, {}", reg_ret, reg_l, reg_r)
                                    .unwrap();
                                writeln!(output, "  seqz {}, {}", reg_ret, reg_ret).unwrap();
                            }
                            _ => {}
                        }

                        //正式分入栈中，会在func_info生成一条sw指令
                        func_info.new_var(output, inst, now_stack_offset);
                        now_stack_offset += value_data.ty().size() as i32;

                        //释放所有用到的寄存器(如果有)
                        free_reg(self, &mut func_info, lhs);
                        free_reg(self, &mut func_info, rhs);
                        free_reg(self, &mut func_info, inst);
                    }
                    // 其他种类暂时遇不到
                    _ => {}
                }
            }
        }

        assert!((now_stack_offset - func_info.stack_size).abs() < 16); //差值应该小于16
                                                                       //恢复栈指针
    }
}
