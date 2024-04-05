use crate::constants::Constants;
use fastrand::Rng;

#[derive(Clone)]
pub struct Fuzzer {
    pub rng: Rng,
    pub constants: Constants,
}

impl Fuzzer {
    pub fn new(rng_seed: u64, constants: Constants) -> Self {
        Self {
            rng: Rng::with_seed(rng_seed),
            constants,
        }
    }

    pub fn with_seed(&self, rng_seed: u64) -> Self {
        Self::new(rng_seed, self.constants.clone())
    }

    pub fn with_constants(&mut self, constants: Constants) -> Self {
        Self::new(self.rng.u64(..), constants)
    }

    pub fn choice<I>(&mut self, iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
    {
        self.rng.choice(iter)
    }

    // Generates random length for a sequence type
    pub fn fuzz_length(&mut self) -> usize {
        let m = 20; // Todo add it as a parameter in constants.lengths
        self.rng.usize(1..m)
    }

    pub fn fuzz_bool(&mut self) -> bool {
        self.rng.bool()
    }

    pub fn fuzz_str(&mut self) -> String {
        match self.rng.choice(&self.constants.str_constants) {
            Some(s) => s.to_string(),
            None => ["A"].repeat(self.rng.usize(1..100)).concat(),
        }
    }

    // Generates a random u8
    pub fn fuzz_u8(&mut self) -> u8 {
        match self.rng.choice(&mut self.constants.u8_constants) {
            Some(c) => *c,
            None => self.rng.u8(..),
        }
    }

    pub fn fuzz_u16(&mut self) -> u16 {
        match self.rng.choice(&mut self.constants.u16_constants) {
            Some(c) => *c,
            None => self.rng.u16(..),
        }
    }

    pub fn fuzz_u32(&mut self) -> u32 {
        match self.rng.choice(&mut self.constants.u32_constants) {
            Some(c) => *c,
            None => self.rng.u32(..),
        }
    }

    pub fn fuzz_u64(&mut self) -> u64 {
        match self.rng.choice(&mut self.constants.u64_constants) {
            Some(c) => *c,
            None => self.rng.u64(..),
        }
    }

    pub fn fuzz_u128(&mut self) -> u128 {
        match self.rng.choice(&mut self.constants.u128_constants) {
            Some(c) => *c,
            None => self.rng.u128(..),
        }
    }

    pub fn fuzz_i8(&mut self) -> i8 {
        match self.rng.choice(&mut self.constants.i8_constants) {
            Some(c) => *c,
            None => self.rng.i8(..),
        }
    }

    pub fn fuzz_i16(&mut self) -> i16 {
        match self.rng.choice(&mut self.constants.i16_constants) {
            Some(c) => *c,
            None => self.rng.i16(..),
        }
    }

    pub fn fuzz_i32(&mut self) -> i32 {
        match self.rng.choice(&mut self.constants.i32_constants) {
            Some(c) => *c,
            None => self.rng.i32(..),
        }
    }

    pub fn fuzz_i64(&mut self) -> i64 {
        match self.rng.choice(&mut self.constants.i64_constants) {
            Some(c) => *c,
            None => self.rng.i64(..),
        }
    }

    pub fn fuzz_i128(&mut self) -> i128 {
        match self.rng.choice(&mut self.constants.i128_constants) {
            Some(c) => *c,
            None => self.rng.i128(..),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_fuzz_length() {
        let mut fuzzer = Fuzzer::new(0, Constants::default());
        let mut lengths = HashSet::new();
        for _ in 0..100 {
            lengths.insert(fuzzer.fuzz_length());
        }
        assert!(lengths.len() > 1);
    }

    #[test]
    fn test_fuzz_bool() {
        let mut fuzzer = Fuzzer::new(0, Constants::default());
        let mut bools = HashSet::new();
        for _ in 0..100 {
            bools.insert(fuzzer.fuzz_bool());
        }
        assert!(bools.len() > 1);
    }

    #[test]
    fn test_fuzz_str() {
        let constants = Constants {
            str_constants: vec!["A".to_string(), "B".to_string()],
            ..Default::default()
        };
        let mut fuzzer = Fuzzer::new(0, constants);
        let mut strings = HashSet::new();
        for _ in 0..100 {
            strings.insert(fuzzer.fuzz_str());
        }
        assert!(strings.len() > 1);
    }

    #[test]
    fn test_with_seed() {
        let mut fuzzer = Fuzzer::new(0, Constants::default());
        let mut fuzzer2 = fuzzer.with_seed(1);
        assert_ne!(fuzzer.rng.u64(..), fuzzer2.rng.u64(..));
    }

    #[test]
    fn test_with_constants() {
        let mut fuzzer = Fuzzer::new(0, Constants::default());
        let fuzzer2 = fuzzer.with_constants(Constants::default());
        assert_eq!(fuzzer.constants, fuzzer2.constants);
    }
}
