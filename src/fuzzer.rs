use std::borrow::BorrowMut;

use fastrand::Rng;
use ink_metadata::{
    ConstructorSpec, InkProject, MessageParamSpec, MessageSpec, Selector,
};
use parity_scale_codec::{Compact as ScaleCompact, Encode};
use scale_info::{
    form::PortableForm, TypeDef, TypeDefArray, TypeDefBitSequence, TypeDefCompact,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

use anyhow::{anyhow, Ok, Result};

pub trait FuzzerTrait {
    fn fuzz_length(&mut self) -> usize;
    fn fuzz_bool(&mut self) -> bool;
    fn fuzz_str(&mut self) -> String;
    fn fuzz_u8(&mut self) -> u8;
    fn fuzz_u16(&mut self) -> u16;
    fn fuzz_u32(&mut self) -> u32;
    fn fuzz_u64(&mut self) -> u64;
    fn fuzz_u128(&mut self) -> u128;
    fn fuzz_i8(&mut self) -> i8;
    fn fuzz_i16(&mut self) -> i16;
    fn fuzz_i32(&mut self) -> i32;
    fn fuzz_i64(&mut self) -> i64;
    fn fuzz_i128(&mut self) -> i128;
    fn choice<I>(&mut self, iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator;
    
}

pub struct Fuzzer<'a> {
    rng: Rng,
    u8_constants: Vec<u8>,
    u16_constants: Vec<u16>,
    u32_constants: Vec<u32>,
    u64_constants: Vec<u64>,
    u128_constants: Vec<u128>,
    i8_constants: Vec<i8>,
    i16_constants: Vec<i16>,
    i32_constants: Vec<i32>,
    i64_constants: Vec<i64>,
    i128_constants: Vec<i128>,
    str_constants: Vec<&'a str>,
}

impl<'a> Default for  Fuzzer<'a> {
    fn default() -> Self {
        let rng_seed = 0;
        Self {
            rng: Rng::with_seed(rng_seed),
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
            str_constants: vec!["UNK"],
        }
    }
}


impl<'a> Fuzzer<'a> {
    pub fn new(rng_seed: u64, u8_constants: Vec<u8>, u16_constants: Vec<u16>, u32_constants: Vec<u32>, u64_constants: Vec<u64>, u128_constants: Vec<u128>, i8_constants: Vec<i8>, i16_constants: Vec<i16>, i32_constants: Vec<i32>, i64_constants: Vec<i64>, i128_constants: Vec<i128>, str_constants: Vec<&'a str>) -> Self {
        Self {
            rng: Rng::with_seed(rng_seed),
            u8_constants,
            u16_constants,
            u32_constants,
            u64_constants,
            u128_constants,
            i8_constants,
            i16_constants,
            i32_constants,
            i64_constants,
            i128_constants,
            str_constants,
        }
    }
}
 

impl<'a> FuzzerTrait for Fuzzer<'a> {
    fn choice<I>(&mut self, iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
    {
        self.rng.choice(iter)
    }

    // Generates random length for a sequence type with bias towards lower numbers
    fn fuzz_length(&mut self) -> usize {
        let m = 20;
        let r = self.rng.usize(1..m);
        m / (r * r)
    }

    fn fuzz_bool(&mut self) -> bool {
        self.rng.bool()
    }

    fn fuzz_str(&mut self) -> String {
        match self.rng.choice(&self.str_constants) {
            Some(s) => s.to_string(),
            None => ["A"].repeat(self.rng.usize(1..100)).concat()
        }
    }

    // Generates a random u8
    fn fuzz_u8(&mut self) -> u8 {
        match self.rng.choice(&self.u8_constants) {
            Some(c) => *c,
            None => self.rng.u8(..),
        }
    }

    fn fuzz_u16(&mut self) -> u16 {
        match self.rng.choice(&self.u16_constants) {
            Some(c) => *c,
            None => self.rng.u16(..),
        }
    }

    fn fuzz_u32(&mut self) -> u32 {
        match self.rng.choice(&self.u32_constants) {
            Some(c) => *c,
            None => self.rng.u32(..),
        }
    }

    fn fuzz_u64(&mut self) -> u64 {
        match self.rng.choice(&self.u64_constants) {
            Some(c) => *c,
            None => self.rng.u64(..),
        }
    }

    fn fuzz_u128(&mut self) -> u128 {
        match self.rng.choice(&self.u128_constants) {
            Some(c) => *c,
            None => self.rng.u128(..),
        }
    }

    fn fuzz_i8(&mut self) -> i8 {
        match self.rng.choice(&self.i8_constants) {
            Some(c) => *c,
            None => self.rng.i8(..),
        }
    }

    fn fuzz_i16(&mut self) -> i16 {
        match self.rng.choice(&self.i16_constants) {
            Some(c) => *c,
            None => self.rng.i16(..),
        }
    }

    fn fuzz_i32(&mut self) -> i32 {
        match self.rng.choice(&self.i32_constants) {
            Some(c) => *c,
            None => self.rng.i32(..),
        }
    }

    fn fuzz_i64(&mut self) -> i64 {
        match self.rng.choice(&self.i64_constants) {
            Some(c) => *c,
            None => self.rng.i64(..),
        }
    }

    fn fuzz_i128(&mut self) -> i128 {
        match self.rng.choice(&self.i128_constants) {
            Some(c) => *c,
            None => self.rng.i128(..),
        }
    }

}