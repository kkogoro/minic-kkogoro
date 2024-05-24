use koopa::ir::entities::ValueData;
use koopa::ir::Value;

use std::collections::HashMap;
const TMP_REG: [&str; 15] = [
    "t0", "t1", "t2", "t3", "t4", "t5", "t6", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7",
];
#[derive(Debug)]
pub struct GenerateAsmInfo {
    pub reg_user: Vec<Option<Value>>,
}

impl GenerateAsmInfo {
    pub fn new() -> Self {
        GenerateAsmInfo {
            reg_user: vec![None; 15],
        }
    }
    pub fn new_reg(&mut self, value: Value) -> Option<String> {
        for (i, user) in self.reg_user.iter_mut().enumerate() {
            if user.is_none() {
                *user = Some(value);
                return Some(TMP_REG[i].to_string());
            }
        }
        None
    }
    pub fn get_reg(&mut self, value: Value) -> Option<String> {
        for (i, user) in self.reg_user.iter().enumerate() {
            if let Some(v) = user {
                if *v == value {
                    return Some(TMP_REG[i].to_string());
                }
            }
        }

        Self::new_reg(self, value)
    }
}
