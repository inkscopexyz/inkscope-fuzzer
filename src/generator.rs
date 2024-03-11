use fastrand::Rng;
use ink_metadata::{
    ConstructorSpec, InkProject, MessageParamSpec, MessageSpec, Selector,
};
use parity_scale_codec::{Compact as ScaleCompact, Encode};
use scale_info::{
    form::PortableForm, TypeDef, TypeDefArray, TypeDefBitSequence, TypeDefCompact,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

use crate::fuzzer::{Fuzzer, FuzzerTrait};
use anyhow::{anyhow, Ok, Result};

enum MessageType<'a> {
    Constructor(&'a ConstructorSpec<PortableForm>),
    Message(&'a MessageSpec<PortableForm>),
}

// Used to fuzz generate a single input data for a constructor or a message
// let generator = Generator::new(&metadata, &selected_message, Option<rng_seed>);
// [methodid][arg0][arg1]
struct Generator<'a> {
    ink_project: &'a InkProject,
    fuzzer: &'a mut Fuzzer<'a>, //TODO: theis may be a generic implementing FuzzerTrait
    selected: MessageType<'a>,
}

// Input Fuzzy Generator for a ConstructorSpec or a MessageSpec.
impl<'a> Generator<'a> {
    pub fn new(
        ink_project: &'a InkProject,
        fuzzer: &'a mut Fuzzer<'a>,
        selected: MessageType<'a>,
    ) -> Self {
        Self {
            ink_project,
            fuzzer,
            selected,
        }
    }

    // Create a new Generator from a label (assuming the label is unique in the  ink_project file)
    pub fn from_label(
        ink_project: &'a InkProject,
        fuzzer: &'a mut Fuzzer<'a>,
        label: &str,
    ) -> Result<Self> {
        let message = ink_project
            .spec()
            .messages()
            .iter()
            .find(|c| c.label() == label);
        if let Some(message) = message {
            return Ok(Self::new(ink_project, fuzzer, MessageType::Message(message)));
        };
        let constructor = ink_project
            .spec()
            .constructors()
            .iter()
            .find(|c| c.label() == label);
        if let Some(constructor) = constructor {
            return Ok(Self::new(
                ink_project,
                fuzzer,
                MessageType::Constructor(constructor),
            ));
        };
        anyhow::bail!("Label not found");
    }

    // Create a new Generator from a selector
    pub fn from_selector(
        ink_project: &'a InkProject,
        fuzzer: &'a mut Fuzzer<'a>,
        selector: &Selector,
    ) -> Result<Self> {
        let message = ink_project
            .spec()
            .messages()
            .iter()
            .find(|c: &&MessageSpec<PortableForm>| c.selector() == selector);
        if let Some(message) = message {
            return Ok(Self::new(ink_project, fuzzer, MessageType::Message(message)));
        };
        let constructor = ink_project
            .spec()
            .constructors()
            .iter()
            .find(|c| c.selector() == selector);
        if let Some(constructor) = constructor {
            return Ok(Self::new(
                ink_project,
                fuzzer,
                MessageType::Constructor(constructor),
            ));
        };
        anyhow::bail!("Selector not found");
    }

    #[inline(always)]
    fn get_typedef(&self, type_id: u32) -> Result<TypeDef<PortableForm>> {
        match self.ink_project.registry().resolve(type_id) {
            Some(type_def) => Ok(type_def.type_def.to_owned()),
            None => Err(anyhow::anyhow!("Type not found")),
        }
    }

    pub fn generate(&mut self) -> Result<Vec<u8>> {
        match self.selected {
            MessageType::Constructor(spec) => self.generate_constructor(spec),
            MessageType::Message(spec) => self.generate_message(spec),
        }
    }

    // Generates a fuzzed constructor encoded input data
    fn generate_constructor(
        &mut self,
        spec: &ConstructorSpec<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = spec.selector().to_bytes().to_vec();
        let mut encoded_args = self.generate_arguments(spec.args())?;
        encoded.append(&mut encoded_args);
        Ok(encoded)
    }
    // Generates a fuzzed message encoded input data
    fn generate_message(&mut self, spec: &MessageSpec<PortableForm>) -> Result<Vec<u8>> {
        let mut encoded = spec.selector().to_bytes().to_vec();
        let mut encoded_args = self.generate_arguments(spec.args())?;
        encoded.append(&mut encoded_args);
        Ok(encoded)
    }

    // Generates a fuzzed arguments encoded input data
    fn generate_arguments(
        &mut self,
        args: &[MessageParamSpec<PortableForm>],
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for arg in args {
            let type_def = self.get_typedef(arg.ty().ty().id)?;
            let mut arg_encoded = self.generate_argument(&type_def)?;
            encoded.append(&mut arg_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed argument encoded input data
    fn generate_argument(&mut self, type_def: &TypeDef<PortableForm>) -> Result<Vec<u8>> {
        match type_def {
            TypeDef::Composite(composite) => self.generate_composite(composite),
            TypeDef::Array(array) => self.generate_array(array),
            TypeDef::Tuple(tuple) => self.generate_tuple(tuple),
            TypeDef::Sequence(sequence) => self.generate_sequence(sequence),
            TypeDef::Variant(variant) => self.generate_variant(variant),
            TypeDef::Primitive(primitive) => self.generate_primitive(primitive),
            TypeDef::Compact(compact) => self.generate_compact(compact),
            TypeDef::BitSequence(bit_sequence) => {
                self.generate_bit_sequence(bit_sequence)
            }
        }
    }

    // Generates a fuzzed encoded data for  composite type, consisting of either named (struct) or unnamed (tuple struct) fields
    fn generate_composite(
        &mut self,
        composite: &TypeDefComposite<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for field in &composite.fields {
            let field_type_def = self.get_typedef(field.ty.id)?;
            let mut field_encoded = self.generate_argument(&field_type_def)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a sized array type like [u8;32].
    // The size is prestablished in the type definition and does not appear in the encoding
    fn generate_array(&mut self, array: &TypeDefArray<PortableForm>) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        //No length is included in the encoding as it is known at decoding
        let param_type_def = self.get_typedef(array.type_param.id)?;
        for _i in 0..array.len {
            let mut param_encoded = self.generate_argument(&param_type_def)?;
            encoded.append(&mut param_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a tuple type like (u8, u16, u32). It is a sequence of prestablished and potentially different types
    // The number of elements and their types are prestablished in the type definition
    fn generate_tuple(&mut self, tuple: &TypeDefTuple<PortableForm>) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for field in &tuple.fields {
            let field_type = self.get_typedef(field.id)?;
            let mut field_encoded = self.generate_argument(&field_type)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a sequence type like [u8]. It is a sequence of fields with the same type with an unknown length.
    // The encoding ioncludes the size and it will be fuzzed .
    fn generate_sequence(
        &mut self,
        sequence: &TypeDefSequence<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        // Fuzz a sequece size and encode it in compact form
        let length = self.fuzzer.fuzz_length() as u32;
        ScaleCompact(length).encode_to(&mut encoded);

        let param_type_def = self.get_typedef(sequence.type_param.id)?;
        for _i in 0..length {
            let mut param_encoded = self.generate_argument(&param_type_def)?;
            encoded.append(&mut param_encoded);
        }
        Ok(encoded)
    }

    // Generates a fuzzed encoded data for a rendomly coosed variant.
    fn generate_variant(
        &mut self,
        variant: &TypeDefVariant<PortableForm>,
    ) -> Result<Vec<u8>> {
        match self.fuzzer.choice(&variant.variants) {
            Some(selected_variant) => {
                // Encode the index of the selected variant
                let mut encoded = selected_variant.index.encode();
                for field in &selected_variant.fields {
                    let field_type = self.get_typedef(field.ty.id)?;
                    let mut field_encoded = self.generate_argument(&field_type)?;
                    encoded.append(&mut field_encoded);
                }
                Ok(encoded)
            }
            None => Err(anyhow::anyhow!("No variant selected")),
        }
    }

    // Generates a fuzzed encoded data for a primitive type like bool, char, u8, u16, u32, u64, u128, u256, i8, i16, i32, i64, i128, i256.
    // Note char is not supported by scale codec
    fn generate_primitive(&mut self, primitive: &TypeDefPrimitive) -> Result<Vec<u8>> {
        match primitive {
            TypeDefPrimitive::Bool => self.generate_bool(),
            TypeDefPrimitive::Char => {
                Err(anyhow::anyhow!("scale codec not implemented for char"))
            }
            TypeDefPrimitive::Str => self.generate_str(),
            TypeDefPrimitive::U8 => self.generate_u8(),
            TypeDefPrimitive::U16 => self.generate_u16(),
            TypeDefPrimitive::U32 => self.generate_u32(),
            TypeDefPrimitive::U64 => self.generate_u64(),
            TypeDefPrimitive::U128 => self.generate_u128(),
            TypeDefPrimitive::U256 => self.generate_u256(),
            TypeDefPrimitive::I8 => self.generate_i8(),
            TypeDefPrimitive::I16 => self.generate_i16(),
            TypeDefPrimitive::I32 => self.generate_i32(),
            TypeDefPrimitive::I64 => self.generate_i64(),
            TypeDefPrimitive::I128 => self.generate_i128(),
            TypeDefPrimitive::I256 => self.generate_i256(),
        }
    }

    // Generates a fuzzed encoded data for a compact type. It is a wrapper around a primitive or a composite type.
    fn generate_compact(
        &mut self,
        compact: &TypeDefCompact<PortableForm>,
    ) -> Result<Vec<u8>> {
        let param_typedef = self.get_typedef(compact.type_param.id)?;
        match param_typedef {
            TypeDef::Primitive(primitive) => self.generate_compact_primitive(&primitive),
            TypeDef::Composite(composite) => self.generate_compact_composite(&composite),
            _ => Err(anyhow::anyhow!(
                "Compact type must be a primitive or a composite type"
            )),
        }
    }

    // Generates a fuzzed encoded data for a compact primitive type like Compact<u8>, Compact<u16>, Compact<u32>, Compact<u64>, Compact<u128>, Compact<u256>.
    fn generate_compact_primitive(
        &mut self,
        primitive: &TypeDefPrimitive,
    ) -> Result<Vec<u8>> {
        match primitive {
            TypeDefPrimitive::U8 => self.generate_compact_u8(),
            TypeDefPrimitive::U16 => self.generate_compact_u16(),
            TypeDefPrimitive::U32 => self.generate_compact_u32(),
            TypeDefPrimitive::U64 => self.generate_compact_u64(),
            TypeDefPrimitive::U128 => self.generate_compact_u128(),
            _ => Err(anyhow::anyhow!(
                "Compact encoding not supported for {:?}",
                primitive
            )),
        }
    }

    // Generates a fuzzed compact encoded data for an u8
    fn generate_compact_u8(&mut self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.fuzzer.fuzz_u8()).encode())
    }

    // Generates a fuzzed compact encoded data for an u16
    fn generate_compact_u16(&mut self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.fuzzer.fuzz_u16()).encode())
    }

    fn generate_compact_u32(&mut self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.fuzzer.fuzz_u32()).encode())
    }

    fn generate_compact_u64(&mut self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.fuzzer.fuzz_u64()).encode())
    }

    fn generate_compact_u128(&mut self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.fuzzer.fuzz_u128()).encode())
    }

    fn generate_compact_composite(
        &self,
        _composite: &TypeDefComposite<PortableForm>,
    ) -> Result<Vec<u8>> {
        todo!("Compact encoding for composite types not supported IMPLEEEMEEENT MEEEEEEEEE!")
    }

    fn generate_bit_sequence(
        &self,
        _bit_sequence: &TypeDefBitSequence<PortableForm>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Bitsequence currently not supported"))
    }

    #[inline(always)]
    fn generate_bool(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_bool().encode())
    }

    #[inline(always)]
    fn generate_str(&mut self) -> Result<Vec<u8>> {
        //TODO: choose for  set of predeined strings extracted from the contract and other sources
        Ok(self.fuzzer.fuzz_str().encode())
    }

    #[inline(always)]
    fn generate_u8(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_u8().encode())
    }

    #[inline(always)]
    fn generate_u16(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_u16().encode())
    }

    #[inline(always)]
    fn generate_u32(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_u32().encode())
    }

    #[inline(always)]
    fn generate_u64(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_u64().encode())
    }

    #[inline(always)]
    fn generate_u128(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_u128().encode())
    }

    #[inline(always)]
    fn generate_u256(&mut self) -> Result<Vec<u8>> {
        //TODO: We can encode a random u256 value
        Err(anyhow::anyhow!("U256 currently not supported"))
    }

    #[inline(always)]
    fn generate_i8(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_i8().encode())
    }

    #[inline(always)]
    fn generate_i16(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_i16().encode())
    }

    #[inline(always)]
    fn generate_i32(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_i32().encode())
    }

    #[inline(always)]
    fn generate_i64(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_i64().encode())
    }

    #[inline(always)]
    fn generate_i128(&mut self) -> Result<Vec<u8>> {
        Ok(self.fuzzer.fuzz_i128().encode())
    }

    #[inline(always)]
    fn generate_i256(&mut self) -> Result<Vec<u8>> {
        //TODO: We can encode a random i256 value
        Err(anyhow::anyhow!("I256 currently not supported"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use drink::ContractBundle;

    use super::*;

    #[test]
    fn test_u8() {
        let contract_path = "./test-contracts/flipper/target/ink/flipper.contract";
        let bundle = ContractBundle::load(&contract_path)
            .unwrap();
        let ink_project = bundle    .transcoder
            .metadata();
        let mut fuzzer = Fuzzer::default();
        let mut generator = Generator::from_label(&ink_project, &mut fuzzer, "new").unwrap();

        let mut corpus = HashSet::new();
        for _i in 0..100 {
            corpus.insert(generator.generate().unwrap());
        }
        assert_eq!(corpus.len(), 2);

        let mut expected_corpus = HashSet::new();
        expected_corpus.insert(vec![155u8, 174, 157, 94, 1]); // new(true)
        expected_corpus.insert(vec![155u8, 174, 157, 94, 0]); // new(false)
        assert_eq!(corpus, expected_corpus);
        println!("{:?}", corpus);
    }
}
