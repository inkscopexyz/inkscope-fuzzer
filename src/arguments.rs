use crate::fuzzer::Fuzzer;
use anyhow::{Ok, Result};
use parity_scale_codec::{Compact as ScaleCompact, Encode};
use scale_info::PortableRegistry;
use scale_info::{
    form::PortableForm, TypeDef, TypeDefArray, TypeDefBitSequence, TypeDefCompact,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

// Used to fuzz generate a single input data for a constructor or a message
pub struct ArgumentsGenerator<'a> {
    registry: &'a PortableRegistry,
    arguments: &'a Vec<TypeDef<PortableForm>>,
}

// Input Fuzzy Generator for a Constructor or Message arguments
impl<'a> ArgumentsGenerator<'a> {
    pub fn new(
        registry: &'a PortableRegistry,
        arguments: &'a Vec<TypeDef<PortableForm>>,
    ) -> Self {
        Self {
            registry,
            arguments,
        }
    }

    #[inline(always)]
    fn get_typedef(&self, type_id: u32) -> Result<TypeDef<PortableForm>> {
        match self.registry.resolve(type_id) {
            Some(type_def) => Ok(type_def.type_def.to_owned()),
            None => Err(anyhow::anyhow!("Type not found")),
        }
    }

    // Generates a fuzzed arguments encoded input data
    pub fn generate(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for type_def in self.arguments {
            //let type_def = self.get_typedef(arg.ty().ty().id)?;
            let mut arg_encoded = self.generate_argument(fuzzer, &type_def)?;
            encoded.append(&mut arg_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed argument encoded input data
    fn generate_argument(
        &self,
        fuzzer: &mut Fuzzer,
        type_def: &TypeDef<PortableForm>,
    ) -> Result<Vec<u8>> {
        match type_def {
            TypeDef::Composite(composite) => self.generate_composite(fuzzer, composite),
            TypeDef::Array(array) => self.generate_array(fuzzer, array),
            TypeDef::Tuple(tuple) => self.generate_tuple(fuzzer, tuple),
            TypeDef::Sequence(sequence) => self.generate_sequence(fuzzer, sequence),
            TypeDef::Variant(variant) => self.generate_variant(fuzzer, variant),
            TypeDef::Primitive(primitive) => self.generate_primitive(fuzzer, primitive),
            TypeDef::Compact(compact) => self.generate_compact(fuzzer, compact),
            TypeDef::BitSequence(bit_sequence) => {
                self.generate_bit_sequence(fuzzer, bit_sequence)
            }
        }
    }

    // Generates a fuzzed encoded data for  composite type, consisting of either named (struct) or unnamed (tuple struct) fields
    fn generate_composite(
        &self,
        fuzzer: &mut Fuzzer,
        composite: &TypeDefComposite<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for field in &composite.fields {
            let field_type_def = self.get_typedef(field.ty.id)?;
            let mut field_encoded = self.generate_argument(fuzzer, &field_type_def)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a sized array type like [u8;32].
    // The size is prestablished in the type definition and does not appear in the encoding
    fn generate_array(
        &self,
        fuzzer: &mut Fuzzer,
        array: &TypeDefArray<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        //No length is included in the encoding as it is known at decoding
        let param_type_def = self.get_typedef(array.type_param.id)?;
        for _i in 0..array.len {
            let mut param_encoded = self.generate_argument(fuzzer, &param_type_def)?;
            encoded.append(&mut param_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a tuple type like (u8, u16, u32).
    // It refers to a sequence of prestablished and potentially different types
    // The number of elements and their types are prestablished in the type definition
    fn generate_tuple(
        &self,
        fuzzer: &mut Fuzzer,
        tuple: &TypeDefTuple<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for field in &tuple.fields {
            let field_type = self.get_typedef(field.id)?;
            let mut field_encoded = self.generate_argument(fuzzer, &field_type)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a sequence type like [u8].
    // It refers to a sequence of fields with the same type with an unknown length.
    // The encoding includes the size and it will be fuzzed .
    fn generate_sequence(
        &self,
        fuzzer: &mut Fuzzer,
        sequence: &TypeDefSequence<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        // Fuzz a sequece size and encode it in compact form
        let length = fuzzer.fuzz_length() as u32;
        ScaleCompact(length).encode_to(&mut encoded);

        let param_type_def = self.get_typedef(sequence.type_param.id)?;
        for _i in 0..length {
            let mut param_encoded = self.generate_argument(fuzzer, &param_type_def)?;
            encoded.append(&mut param_encoded);
        }
        Ok(encoded)
    }

    fn generate_variant(
        &self,
        fuzzer: &mut Fuzzer,
        variant: &TypeDefVariant<PortableForm>,
    ) -> Result<Vec<u8>> {
        match fuzzer.choice(&variant.variants) {
            Some(selected_variant) => {
                // Encode the index of the selected variant
                let mut encoded = selected_variant.index.encode();
                for field in &selected_variant.fields {
                    let field_type = self.get_typedef(field.ty.id)?;
                    let mut field_encoded =
                        self.generate_argument(fuzzer, &field_type)?;
                    encoded.append(&mut field_encoded);
                }
                Ok(encoded)
            }
            None => Err(anyhow::anyhow!("No variant selected")),
        }
    }

    // Generates a fuzzed encoded data for a primitive type like bool, char, u8, u16, u32, u64, u128, u256, i8, i16, i32, i64, i128, i256.
    // Note char is not supported by scale codec
    fn generate_primitive(
        &self,
        fuzzer: &mut Fuzzer,
        primitive: &TypeDefPrimitive,
    ) -> Result<Vec<u8>> {
        match primitive {
            TypeDefPrimitive::Bool => self.generate_bool(fuzzer),
            TypeDefPrimitive::Char => {
                Err(anyhow::anyhow!("scale codec not implemented for char"))
            }
            TypeDefPrimitive::Str => self.generate_str(fuzzer),
            TypeDefPrimitive::U8 => self.generate_u8(fuzzer),
            TypeDefPrimitive::U16 => self.generate_u16(fuzzer),
            TypeDefPrimitive::U32 => self.generate_u32(fuzzer),
            TypeDefPrimitive::U64 => self.generate_u64(fuzzer),
            TypeDefPrimitive::U128 => self.generate_u128(fuzzer),
            TypeDefPrimitive::U256 => self.generate_u256(fuzzer),
            TypeDefPrimitive::I8 => self.generate_i8(fuzzer),
            TypeDefPrimitive::I16 => self.generate_i16(fuzzer),
            TypeDefPrimitive::I32 => self.generate_i32(fuzzer),
            TypeDefPrimitive::I64 => self.generate_i64(fuzzer),
            TypeDefPrimitive::I128 => self.generate_i128(fuzzer),
            TypeDefPrimitive::I256 => self.generate_i256(fuzzer),
        }
    }

    fn generate_compact(
        &self,
        fuzzer: &mut Fuzzer,
        compact: &TypeDefCompact<PortableForm>,
    ) -> Result<Vec<u8>> {
        let param_typedef = self.get_typedef(compact.type_param.id)?;
        match param_typedef {
            TypeDef::Primitive(primitive) => {
                self.generate_compact_primitive(fuzzer, &primitive)
            }
            TypeDef::Composite(composite) => {
                self.generate_compact_composite(fuzzer, &composite)
            }
            _ => Err(anyhow::anyhow!(
                "Compact type must be a primitive or a composite type"
            )),
        }
    }

    fn generate_compact_primitive(
        &self,
        fuzzer: &mut Fuzzer,
        primitive: &TypeDefPrimitive,
    ) -> Result<Vec<u8>> {
        match primitive {
            TypeDefPrimitive::U8 => self.generate_compact_u8(fuzzer),
            TypeDefPrimitive::U16 => self.generate_compact_u16(fuzzer),
            TypeDefPrimitive::U32 => self.generate_compact_u32(fuzzer),
            TypeDefPrimitive::U64 => self.generate_compact_u64(fuzzer),
            TypeDefPrimitive::U128 => self.generate_compact_u128(fuzzer),
            _ => Err(anyhow::anyhow!(
                "Compact encoding not supported for {:?}",
                primitive
            )),
        }
    }

    fn generate_compact_u8(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(ScaleCompact(fuzzer.fuzz_u8()).encode())
    }

    fn generate_compact_u16(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(ScaleCompact(fuzzer.fuzz_u16()).encode())
    }

    fn generate_compact_u32(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(ScaleCompact(fuzzer.fuzz_u32()).encode())
    }

    fn generate_compact_u64(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(ScaleCompact(fuzzer.fuzz_u64()).encode())
    }

    fn generate_compact_u128(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(ScaleCompact(fuzzer.fuzz_u128()).encode())
    }

    fn generate_compact_composite(
        &self,
        _fuzzer: &mut Fuzzer,
        _composite: &TypeDefComposite<PortableForm>,
    ) -> Result<Vec<u8>> {
        todo!("Compact encoding for composite types not supported IMPLEEEMEEENT MEEEEEEEEE!")
    }

    fn generate_bit_sequence(
        &self,
        _fuzzer: &mut Fuzzer,
        _bit_sequence: &TypeDefBitSequence<PortableForm>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Bitsequence currently not supported"))
    }

    #[inline(always)]
    fn generate_bool(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_bool().encode())
    }

    #[inline(always)]
    fn generate_str(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        //TODO: choose for  set of predeined strings extracted from the contract and other sources
        Ok(fuzzer.fuzz_str().encode())
    }

    #[inline(always)]
    fn generate_u8(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_u8().encode())
    }

    #[inline(always)]
    fn generate_u16(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_u16().encode())
    }

    #[inline(always)]
    fn generate_u32(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_u32().encode())
    }

    #[inline(always)]
    fn generate_u64(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_u64().encode())
    }

    #[inline(always)]
    fn generate_u128(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_u128().encode())
    }

    #[inline(always)]
    fn generate_u256(&self, _fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        //TODO: We can encode a random u256 value
        Err(anyhow::anyhow!("U256 currently not supported"))
    }

    #[inline(always)]
    fn generate_i8(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_i8().encode())
    }

    #[inline(always)]
    fn generate_i16(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_i16().encode())
    }

    #[inline(always)]
    fn generate_i32(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_i32().encode())
    }

    #[inline(always)]
    fn generate_i64(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_i64().encode())
    }

    #[inline(always)]
    fn generate_i128(&self, fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        Ok(fuzzer.fuzz_i128().encode())
    }

    #[inline(always)]
    fn generate_i256(&self, _fuzzer: &mut Fuzzer) -> Result<Vec<u8>> {
        //TODO: We can encode a random i256 value
        Err(anyhow::anyhow!("I256 currently not supported"))
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::Constants;

    use super::*;
    use drink::ContractBundle;
    use std::collections::HashSet;

    #[test]
    fn test_generate() {
        // A rather big unittest that tests we generate all possible new() calls in the flipper contract
        let contract_path = "./test-contracts/flipper/target/ink/flipper.contract";
        let bundle = ContractBundle::load(&contract_path).unwrap();
        let ink_project = bundle.transcoder.metadata();
        let mut fuzzer = Fuzzer::new(0, Constants::default());
        let selected = ink_project
            .spec()
            .messages()
            .iter()
            .find(|m| m.label() == "new")
            .unwrap()
            .args()
            .iter()
            .map(|arg| {
                ink_project
                    .registry()
                    .resolve(arg.ty().ty().id)
                    .unwrap()
                    .type_def
                    .clone()
            })
            .collect::<Vec<_>>();

        let generator = ArgumentsGenerator::new(ink_project.registry(), &selected);
        // let mut generator =
        //     ArgumentsGenerator::from_label(&ink_project, &mut fuzzer, "new").unwrap();

        let mut corpus = HashSet::new();
        for _i in 0..100 {
            corpus.insert(generator.generate(&mut fuzzer).unwrap());
        }
        assert_eq!(corpus.len(), 2);

        let mut expected_corpus = HashSet::new();
        expected_corpus.insert(vec![155u8, 174, 157, 94, 1]); // new(true)
        expected_corpus.insert(vec![155u8, 174, 157, 94, 0]); // new(false)
        assert_eq!(corpus, expected_corpus);
    }
}
