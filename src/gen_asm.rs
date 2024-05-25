//! 根据内存形式 Koopa IR 生成汇编
use std::fmt::write;
use std::{fs::File, io::Write};

use crate::ds_for_asm::check_i12;
use crate::ds_for_asm::GenerateAsmInfo;
use crate::ds_for_asm::UserKind;
use koopa::ir::values::*;
use koopa::ir::*;
pub trait GenerateAsm {
    type GenerateResult;
    fn generate(&self, output: &mut File, program_info: &Program) -> Self::GenerateResult;
}

fn get_reg(
    output: &mut File,
    func_data: &koopa::ir::FunctionData,
    func_info: &mut GenerateAsmInfo,
    value: Value,
    program_info: &Program,
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
        _ => func_info.get_reg(output, value, program_info), //expr
    }
}

///释放寄存器，TODO:先都释放掉，后面加spill再把用到这个的都删掉
fn free_reg(func_data: &koopa::ir::FunctionData, func_info: &mut GenerateAsmInfo, value: Value) {
    if value.is_global() {
        //全局变量直接释放，不要再表中查询
        func_info.free_reg(UserKind::Val(value));
        return;
    }
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

fn store_by_offset(output: &mut File, func_info: &mut GenerateAsmInfo, reg: &str, offset: i32) {
    if check_i12(offset) {
        writeln!(output, "  sw {}, {}(sp)", reg, offset).unwrap();
    } else {
        let reg_addr = func_info.get_reg_i32(output, offset);
        writeln!(output, "  add {}, sp, {}", reg_addr, reg_addr).unwrap();
        writeln!(output, "  sw {}, 0({})", reg, reg_addr).unwrap();
        func_info.free_reg(UserKind::Tmpi32(offset));
    }
}

fn load_by_offset(output: &mut File, func_info: &mut GenerateAsmInfo, reg: &str, offset: i32) {
    if check_i12(offset) {
        writeln!(output, "  lw {}, {}(sp)", reg, offset).unwrap();
    } else {
        let reg_addr = func_info.get_reg_i32(output, offset);
        writeln!(output, "  add {}, sp, {}", reg_addr, reg_addr).unwrap();
        writeln!(output, "  lw {}, 0({})", reg, reg_addr).unwrap();
        func_info.free_reg(UserKind::Tmpi32(offset));
    }
}

/// 为Program实现GenerateAsm trait
impl GenerateAsm for Program {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, program_info: &Program) {
        for &value in self.inst_layout() {
            let data = self.borrow_value(value);
            let name = &data.name().as_ref().unwrap()[1..];
            //info.insert_value(value, name.into());
            writeln!(output, "  .data").unwrap();
            writeln!(output, "  .globl {}", name).unwrap();
            writeln!(output, "{}:", name).unwrap();
            value.generate(output, program_info);
        }
        // 遍历函数列表
        for &func in self.func_layout() {
            self.func(func).generate(output, program_info);
        }
    }
}

impl GenerateAsm for koopa::ir::Value {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, program_info: &Program) {
        let data = program_info.borrow_value(*self);
        match data.kind() {
            ValueKind::GlobalAlloc(v) => {
                let x = v.init();
                x.generate(output, program_info);
            }
            ValueKind::Aggregate(v) => {
                for &elem in v.elems() {
                    elem.generate(output, program_info);
                }
            }
            ValueKind::Integer(v) => {
                writeln!(output, "  .word {}", v.value()).unwrap();
            }
            ValueKind::ZeroInit(_) => {
                writeln!(output, "  .zero {}", data.ty().size()).unwrap();
            }
            _ => {}
        }
    }
}

/// 为FunctionData实现GenerateAsm trait
impl GenerateAsm for koopa::ir::FunctionData {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, program_info: &Program) {
        //跳过声明
        if self.layout().entry_bb().is_none() {
            return;
        }

        writeln!(output, "  .text").unwrap();
        writeln!(output, "  .globl {}", &self.name()[1..]).unwrap();
        writeln!(output, "{}:", &self.name()[1..]).unwrap();

        let mut func_info = GenerateAsmInfo::new();

        let mut ra_size = 0; //ra大小 0 or 4
        let mut local_var_size = 0; //局部变量空间大小
        let mut param_size = 0; //参数空间大小
                                /*
                                _______
                                   ra
                                -------
                                局部区域
                                -------
                                参数区域
                                _______

                                 */

        //计算各部分大小
        for (&bb, node) in self.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                let value_type = value_data.ty();
                if let ValueKind::Call(call_inst) = value_data.kind() {
                    let mut now_params_size = 0;
                    let mut now_params_cnt = 0;
                    ra_size = 4; //有函数调用，需要保存ra
                    for &param in call_inst.args() {
                        now_params_cnt += 1;
                        if now_params_cnt > 8 {
                            now_params_size += self.dfg().value(param).ty().size() as i32;
                        }
                    }
                    if now_params_size > param_size {
                        param_size = now_params_size;
                    }
                }
                if value_type.is_unit() {
                    //没有返回值，不需要分配栈空间
                    continue;
                }
                local_var_size += value_type.size() as i32;
            }
        }

        //计算sp偏移量并对齐16
        func_info.stack_size = ra_size + local_var_size + param_size;
        func_info.stack_size = (func_info.stack_size + 15) & !15; //check?

        //取回参数，此时栈指针还没有移动
        {
            let mut now_params_offset = 0;
            for (i, &param) in self.params().iter().enumerate() {
                let reg_param = get_reg(output, self, &mut func_info, param, program_info);
                if i <= 7 {
                    //参数个数小于等于8个，从a0-a7读，这里直接把对应寄存器的占用情况设置成对应参数
                    func_info.set_reg(&("a".to_owned() + &i.to_string()), param)
                    //注意，寄存器里的东西读完就不要再访问了，我们的前端保证一定会先把他们备份一遍
                } else {
                    //超过8个从栈上读取
                    //直接把参数对应偏移量设置成对应位置
                    func_info.set_offset(param, now_params_offset + func_info.stack_size);
                    now_params_offset += self.dfg().value(param).ty().size() as i32;
                }
                free_reg(self, &mut func_info, param);
            }
        }

        //移动栈指针
        func_info.set_sp(output);

        //计算ra偏移量，是否保存ra是由ra_size决定的
        let ra_offset = local_var_size + param_size;
        if ra_size > 0 {
            //保存ra
            store_by_offset(output, &mut func_info, "ra", ra_offset);
        }

        //为每个元素分配栈偏移量，从参数区域正上方开始
        let mut now_stack_offset = param_size;

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
                    ValueKind::Call(call_inst) => {
                        let mut now_params_offset = 0;
                        for (i, &param) in call_inst.args().iter().enumerate() {
                            let reg_param =
                                get_reg(output, self, &mut func_info, param, program_info);
                            if i <= 7 {
                                //参数个数小于等于8个，存在a0-a7
                                writeln!(output, "  mv a{}, {}", i, reg_param).unwrap();
                            } else {
                                //存栈上
                                store_by_offset(
                                    output,
                                    &mut func_info,
                                    &reg_param,
                                    now_params_offset,
                                );

                                now_params_offset += self.dfg().value(param).ty().size() as i32;
                            }
                            free_reg(self, &mut func_info, param);
                        }
                        writeln!(
                            output,
                            "  call {}",
                            &(program_info.func(call_inst.callee()).name())[1..]
                        )
                        .unwrap();

                        if value_data.ty().is_unit() {
                            //没有返回值，不需要设置返回值
                            continue;
                        }

                        func_info.set_reg("a0", inst); //把a0的所有者给inst

                        //正式分入栈中，会在func_info生成一条sw指令
                        func_info.new_var(output, inst, now_stack_offset, program_info);
                        free_reg(self, &mut func_info, inst); //TODO

                        now_stack_offset += value_data.ty().size() as i32;
                    }
                    ValueKind::Branch(br_inst) => {
                        let cond = br_inst.cond();
                        let true_bb_name = self.dfg().bb(br_inst.true_bb()).name().clone().unwrap();
                        let false_bb_name =
                            self.dfg().bb(br_inst.false_bb()).name().clone().unwrap();
                        let reg_cond = get_reg(output, self, &mut func_info, cond, program_info);
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
                                        let reg_ret = get_reg(
                                            output,
                                            self,
                                            &mut func_info,
                                            ret_val,
                                            program_info,
                                        );
                                        writeln!(output, "  mv a0, {}", reg_ret).unwrap();

                                        free_reg(self, &mut func_info, ret_val);
                                    }
                                }
                            }
                            None => {}
                        }
                        if ra_size > 0 {
                            //恢复ra
                            let addr_reg = func_info.get_reg_i32(output, ra_offset);
                            writeln!(output, "  li {}, {}", addr_reg, ra_offset).unwrap();
                            writeln!(output, "  add {}, sp, {}", addr_reg, addr_reg).unwrap();
                            writeln!(output, "  lw ra, 0({})", addr_reg).unwrap();
                            func_info.free_reg(UserKind::Tmpi32(ra_offset));
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

                        let reg_ret = get_reg(output, self, &mut func_info, inst, program_info);
                        if addr.is_global() {
                            //全局变量
                            let addr_reg = func_info.get_reg(output, addr, program_info);
                            writeln!(output, "  lw {}, 0({})", reg_ret, addr_reg).unwrap();
                            free_reg(self, &mut func_info, addr);
                        } else {
                            let offset = *func_info.name_to_offset.get(&addr).unwrap();
                            load_by_offset(output, &mut func_info, &reg_ret, offset);
                        }
                        func_info.new_var(output, inst, now_stack_offset, program_info);
                        now_stack_offset += value_data.ty().size() as i32;
                        free_reg(self, &mut func_info, inst);
                    }
                    ValueKind::Store(store_inst) => {
                        // 处理 store 指令
                        let addr = store_inst.dest();
                        let value = store_inst.value();
                        let reg_val = get_reg(output, self, &mut func_info, value, program_info);
                        //TODO:这里应该由IR保证保证store的来源是局部符号
                        if addr.is_global() {
                            let addr_reg = func_info.get_reg(output, addr, program_info);
                            writeln!(output, "  sw {}, 0({})", reg_val, addr_reg).unwrap();
                            free_reg(self, &mut func_info, addr);
                        } else {
                            let offset = *func_info.name_to_offset.get(&addr).unwrap();
                            store_by_offset(output, &mut func_info, &reg_val, offset);
                        }
                        free_reg(self, &mut func_info, value);
                    }
                    ValueKind::Binary(bin_inst) => {
                        // 处理二元运算
                        let lhs = bin_inst.lhs();
                        let rhs = bin_inst.rhs();

                        let reg_l = get_reg(output, self, &mut func_info, lhs, program_info);

                        let reg_r = get_reg(output, self, &mut func_info, rhs, program_info);

                        let reg_ret = get_reg(output, self, &mut func_info, inst, program_info); //指令ID对应结果

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
                        func_info.new_var(output, inst, now_stack_offset, program_info);
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
        //恢复栈指针
    }
}
