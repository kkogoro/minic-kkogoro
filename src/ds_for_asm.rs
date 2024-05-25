use core::panic;
use koopa::ir::entities::ValueData;
use koopa::ir::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const TMP_REG: [&str; 15] = [
    "t0", "t1", "t2", "t3", "t4", "t5", "t6", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7",
];

//ture if val is in range [-2048, 2047]
pub fn check_i12(val: i32) -> bool {
    val >= -2048 && val <= 2047
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserKind {
    Val(Value),
    Tmpi32(i32),
}

#[derive(Debug)]
pub struct GenerateAsmInfo {
    pub reg_user: Vec<Option<UserKind>>,
    pub name_to_offset: HashMap<Value, i32>,
    pub stack_size: i32,
}

impl GenerateAsmInfo {
    pub fn new() -> Self {
        GenerateAsmInfo {
            reg_user: vec![None; 15],
            name_to_offset: HashMap::new(),
            stack_size: 0,
        }
    }
    ///将value的对sp偏移量设置为offset
    pub fn set_offset(&mut self, value: Value, offset: i32) {
        self.name_to_offset.insert(value, offset);
    }
    ///在栈上分配alloc
    pub fn alloc(&mut self, output: &mut File, value: Value, offset: i32) {
        self.set_offset(value, offset);
    }
    ///在栈上分配变量，并且将inst的运算结果存到栈上
    pub fn new_var(&mut self, output: &mut File, value: Value, offset: i32) {
        self.set_offset(value, offset);
        let reg = self.get_reg(output, value);
        if check_i12(offset) {
            writeln!(output, "  sw {}, {}(sp)", reg, offset).unwrap();
        } else {
            let addr_reg = self.get_reg_i32(output, offset);
            writeln!(output, "  add {}, sp, {}", addr_reg, addr_reg).unwrap();
            writeln!(output, "  sw {}, 0({})", reg, addr_reg).unwrap();
            self.free_reg(UserKind::Tmpi32(offset));
        }
    }
    //TODO: 把寄存器当cache用，spill很好做，维护好寄存器上存没存变量，如果存了直接从寄存器读，没存从内存读，每次踢出写回内存
    pub fn set_sp(&mut self, output: &mut File) {
        let delta = -self.stack_size;
        if check_i12(delta) {
            writeln!(output, "  addi sp, sp, {}", delta).unwrap();
            return;
        } else {
            let reg = self.get_reg_i32(output, delta);

            writeln!(output, "  add sp, sp, {}", reg).unwrap();

            self.free_reg(UserKind::Tmpi32(delta));
        }
    }
    pub fn reset_sp(&mut self, output: &mut File) {
        let delta = self.stack_size;
        if check_i12(delta) {
            writeln!(output, "  addi sp, sp, {}", delta).unwrap();
            return;
        } else {
            let reg = self.get_reg_i32(output, delta);

            writeln!(output, "  add sp, sp, {}", reg).unwrap();

            self.free_reg(UserKind::Tmpi32(delta));
        }
    }
    //在可用临时寄存器中分配一个寄存器
    fn new_tmp_reg(&mut self, value: UserKind) -> String {
        for (i, user) in self.reg_user.iter_mut().enumerate() {
            if user.is_none() {
                *user = Some(value);
                return TMP_REG[i].to_string();
            }
        }
        panic!("寄存器不足");
    }
    // 释放寄存器
    pub fn free_reg(&mut self, free_user: UserKind) {
        for iter in self.reg_user.iter_mut() {
            if let Some(user) = iter {
                if *user == free_user {
                    *iter = None;
                    return;
                }
            }
        }
        panic!("user不占有任何寄存器!");
    }
    //专门为超过i12的偏移量分配寄存器
    pub fn get_reg_i32(&mut self, output: &mut File, inum: i32) -> String {
        let reg = self.new_tmp_reg(UserKind::Tmpi32(inum));
        writeln!(output, "  li {}, {}", reg, inum).unwrap();
        reg
    }
    // 获取value对应的寄存器
    pub fn get_reg(&mut self, output: &mut File, value: Value) -> String {
        //查询是否已经有reg
        for (i, user) in self.reg_user.iter().enumerate() {
            if let Some(v) = user {
                if *v == UserKind::Val(value) {
                    return TMP_REG[i].to_string();
                }
            }
        }
        //还在栈中，分配一个寄存器
        let reg = Self::new_tmp_reg(self, UserKind::Val(value));
        let opt_offset = self.name_to_offset.get(&value);
        match opt_offset {
            Some(offset) => {
                //已经分配入栈，直接读出来
                let offset = *offset;
                if check_i12(offset) {
                    //偏移量在i12
                    writeln!(output, "  lw {}, {}(sp)", reg, offset).unwrap();
                } else {
                    //偏移量超出i12
                    let reg_inum = self.get_reg_i32(output, offset);
                    writeln!(output, "  add {}, sp, {}", reg_inum, reg_inum).unwrap();
                    writeln!(output, "  lw {}, 0({})", reg, reg_inum).unwrap();
                    self.free_reg(UserKind::Tmpi32(offset));
                }
                reg
            }
            None => {
                //尚未分配入栈，不生成读栈代码
                reg
            }
        }
    }
}
