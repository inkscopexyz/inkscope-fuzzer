use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Constants {
    pub u8_constants: Vec<u8>,
    pub u16_constants: Vec<u16>,
    pub u32_constants: Vec<u32>,
    pub u64_constants: Vec<u64>,
    pub u128_constants: Vec<u128>,
    pub i8_constants: Vec<i8>,
    pub i16_constants: Vec<i16>,
    pub i32_constants: Vec<i32>,
    pub i64_constants: Vec<i64>,
    pub i128_constants: Vec<i128>,
    pub str_constants: Vec<String>,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            u8_constants: vec![0, 1, 2, 100, u8::MAX],
            u16_constants: vec![0, 1, 2, 100, u16::MAX],
            u32_constants: vec![0, 1, 2, 100, u32::MAX],
            u64_constants: vec![0, 1, 2, 100, u64::MAX],
            u128_constants: vec![0, 1, 2, u128::MAX],
            i8_constants: vec![i8::MIN, -1, 0, 1, i8::MAX],
            i16_constants: vec![i16::MIN, -1, 0, 1, i16::MAX],
            i32_constants: vec![i32::MIN, -1, 0, 1, i32::MAX],
            i64_constants: vec![i64::MIN, -1, 0, 1, i64::MAX],
            i128_constants: vec![i128::MIN, -1, 0, 1, i128::MAX],
            str_constants: vec!["UNK".into()],
        }
    }
}
