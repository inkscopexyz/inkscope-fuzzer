use std::collections::HashSet;

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Constants {
    pub u8_constants: HashSet<u8>,
    pub u16_constants: HashSet<u16>,
    pub u32_constants: HashSet<u32>,
    pub u64_constants: HashSet<u64>,
    pub u128_constants: HashSet<u128>,
    pub i8_constants: HashSet<i8>,
    pub i16_constants: HashSet<i16>,
    pub i32_constants: HashSet<i32>,
    pub i64_constants: HashSet<i64>,
    pub i128_constants: HashSet<i128>,
    pub str_constants: HashSet<String>,
    pub account_id_constants: HashSet<[u8; 32]>,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            u8_constants: [0, 1, 2, 100, u8::MAX].into(),
            u16_constants: [0, 1, 2, 100, u16::MAX].into(),
            u32_constants: [0, 1, 2, 100, u32::MAX].into(),
            u64_constants: [0, 1, 2, 100, u64::MAX].into(),
            u128_constants: [0, 1, 2, u128::MAX].into(),
            i8_constants: [i8::MIN, -1, 0, 1, i8::MAX].into(),
            i16_constants: [i16::MIN, -1, 0, 1, i16::MAX].into(),
            i32_constants: [i32::MIN, -1, 0, 1, i32::MAX].into(),
            i64_constants: [i64::MIN, -1, 0, 1, i64::MAX].into(),
            i128_constants: [i128::MIN, -1, 0, 1, i128::MAX].into(),
            str_constants: ["UNK".into()].into(),
            account_id_constants: [[0u8; 32], [1u8; 32]].into(),
        }
    }
}

impl Constants {
    fn extend(&mut self, other: &Self) {
        self.u8_constants.extend(&other.u8_constants);
        self.u16_constants.extend(&other.u16_constants);
        self.u32_constants.extend(&other.u32_constants);
        self.u64_constants.extend(&other.u64_constants);
        self.u128_constants.extend(&other.u128_constants);
        self.i8_constants.extend(&other.i8_constants);
        self.i16_constants.extend(&other.i16_constants);
        self.i32_constants.extend(&other.i32_constants);
        self.i64_constants.extend(&other.i64_constants);
        self.i128_constants.extend(&other.i128_constants);
        self.str_constants.extend(other.str_constants.clone());
        self.account_id_constants.extend(&other.account_id_constants);
    }
}
