use crate::fuzzer::Fuzzer;
use anyhow::{
    Ok,
    Result,
};
use parity_scale_codec::{
    Compact as ScaleCompact,
    Encode,
};
use scale_info::{
    form::PortableForm,
    PortableRegistry,
    TypeDef,
    TypeDefArray,
    TypeDefBitSequence,
    TypeDefCompact,
    TypeDefComposite,
    TypeDefPrimitive,
    TypeDefSequence,
    TypeDefTuple,
    TypeDefVariant,
};

// Used to fuzz generate a single input data for a constructor or a message
pub struct Generator<'a> {
    registry: &'a PortableRegistry,
}

// Input Fuzzy Generator for a Constructor or Message arguments
impl<'a> Generator<'a> {
    pub fn new(registry: &'a PortableRegistry) -> Self {
        Self { registry }
    }

    #[inline(always)]
    fn get_typedef(&self, type_id: u32) -> Result<TypeDef<PortableForm>> {
        match self.registry.resolve(type_id) {
            Some(type_def) => Ok(type_def.type_def.to_owned()),
            None => Err(anyhow::anyhow!("Type not found")),
        }
    }

    // Generates a fuzzed arguments encoded input data
    pub fn generate(
        &self,
        fuzzer: &mut Fuzzer,
        arguments: &Vec<TypeDef<PortableForm>>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for type_def in arguments {
            // let type_def = self.get_typedef(arg.ty().ty().id)?;
            let mut arg_encoded = self.generate_argument(fuzzer, type_def)?;
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

    // Generates a fuzzed encoded data for  composite type, consisting of either named
    // (struct) or unnamed (tuple struct) fields
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
    // The size is prestablished in the type definition and does not appear in the
    // encoding
    fn generate_array(
        &self,
        fuzzer: &mut Fuzzer,
        array: &TypeDefArray<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        // No length is included in the encoding as it is known at decoding
        let param_type_def = self.get_typedef(array.type_param.id)?;

        // If the array is compatible with the accountId type we use the accountId
        // constants
        if array.len == 32 && param_type_def == TypeDef::Primitive(TypeDefPrimitive::U8) {
            encoded.append(&mut fuzzer.fuzz_account_id().encode())
        } else {
            // Otherwise we generate the array with the fuzzed values
            for _i in 0..array.len {
                let mut param_encoded =
                    self.generate_argument(fuzzer, &param_type_def)?;
                encoded.append(&mut param_encoded);
            }
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

    // Generates a fuzzed encoded data for a primitive type like bool, char, u8, u16, u32,
    // u64, u128, u256, i8, i16, i32, i64, i128, i256. Note char is not supported by
    // scale codec
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
            _ => {
                Err(anyhow::anyhow!(
                    "Compact type must be a primitive or a composite type"
                ))
            }
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
            _ => {
                Err(anyhow::anyhow!(
                    "Compact encoding not supported for {:?}",
                    primitive
                ))
            }
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
        // TODO: choose for  set of predeined strings extracted from the contract and
        // other sources
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
        // TODO: We can encode a random u256 value
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
        // TODO: We can encode a random i256 value
        Err(anyhow::anyhow!("I256 currently not supported"))
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::Constants;

    use super::*;
    use parity_scale_codec::{
        Compact,
        Decode,
    };
    use scale_info::{
        build::{
            Fields,
            Variants,
        },
        Path,
        PortableRegistryBuilder,
        Type,
    };

    #[test]
    fn test_generate_u8() {
        // make simple registry with a single u8 type
        let mut builder = PortableRegistryBuilder::new();
        let u8_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U8, vec![]);
        let u8_type_id = builder.register_type(u8_type);
        let registry = builder.finish();

        // build a generator for that registry
        let generator = Generator::new(&registry);
        // A constant fuzzer containing only 0x41 as u8 constants
        let mut fuzzer = Fuzzer::new(
            0,
            Constants {
                u8_constants: vec![0x41],
                ..Default::default()
            },
        );

        // get the same type we just register from the registry
        let ty = registry.resolve(u8_type_id).unwrap().type_def.clone();

        // Generate a fuzzed encoded data for it
        let encoded = generator.generate(&mut fuzzer, &vec![ty]).unwrap();

        // Encode it by hand
        let mut raw_encoded = vec![];
        (0x41u8).encode_to(&mut raw_encoded);

        // Check that the generated encoded data is the same as the one we encoded by hand
        assert_eq!(encoded, raw_encoded);
    }

    #[test]
    fn test_vector_of_u32() {
        // Sequence: A collection of same-typed values is encoded, prefixed with a compact
        // encoding of the number of items, followed by each item's encoding
        // concatenated in turn.

        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);

        // Sequence
        let vec_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(u32_type_id.into()),
            vec![],
        );
        let vec_u32_type_id = builder.register_type(vec_u32_type);
        let registry = builder.finish();

        // build a generator for that registry
        let generator = Generator::new(&registry);
        // A constant fuzzer containing only 0x41 as u8 constants
        let mut fuzzer = Fuzzer::new(
            0,
            Constants {
                u32_constants: vec![0x41],
                ..Default::default()
            },
        );

        // get the same type we just register from the registry
        let ty = registry.resolve(vec_u32_type_id).unwrap().type_def.clone();

        // Generate a fuzzed encoded data for it
        let encoded = generator.generate(&mut fuzzer, &vec![ty]).unwrap();
        let parsing_pointer = &mut &encoded[..];

        // Decode it by hand
        let len = <Compact<u32>>::decode(parsing_pointer).unwrap().into();
        assert!(len > 0); // At least one element with this seed
        for _ in 0..len {
            // All elements are taken from the constants (only 0x41)
            assert_eq!(u32::decode(parsing_pointer).unwrap(), 0x41);
        }
    }

    #[test]
    fn test_array_of_u32() {
        // Array: A fixed-size collection of same-typed values is encoded, followed by
        // each item's encoding concatenated in turn.

        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);

        // Array
        let array_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefArray::new(3, u32_type_id.into()),
            vec![],
        );
        let array_u32_type_id = builder.register_type(array_u32_type);
        let registry = builder.finish();

        // build a generator for that registry
        let generator = Generator::new(&registry);
        // A constant fuzzer containing only 0x41 as u8 constants
        let mut fuzzer = Fuzzer::new(
            0,
            Constants {
                u32_constants: vec![0x41],
                ..Default::default()
            },
        );

        // get the same type we just register from the registry
        let ty = registry
            .resolve(array_u32_type_id)
            .unwrap()
            .type_def
            .clone();

        // Generate a fuzzed encoded data for it
        let encoded = generator.generate(&mut fuzzer, &vec![ty]).unwrap();
        let parsing_pointer = &mut &encoded[..];

        // Decode it by hand
        for _ in 0..3 {
            // All elements are taken from the constants (only 0x41)
            assert_eq!(u32::decode(parsing_pointer).unwrap(), 0x41);
        }

        assert!(u32::decode(parsing_pointer).is_err()); // No more elements
    }

    #[test]
    fn test_tuple() {
        // Tuple: A fixed-size collection of potentially different-typed values is
        // encoded, followed by each item's encoding concatenated in turn.

        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);
        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let u64_type_id = builder.register_type(u64_type);

        // Tuple
        let tuple_type = Type::new(
            Path::default(),
            vec![],
            TypeDefTuple::new_portable(vec![u32_type_id.into(), u64_type_id.into()]),
            vec![],
        );
        let tuple_type_id = builder.register_type(tuple_type);
        let registry = builder.finish();

        // build a generator for that registry
        let generator = Generator::new(&registry);
        // A constant fuzzer containing only 0x41 as u8 constants
        let mut fuzzer = Fuzzer::new(
            0,
            Constants {
                u32_constants: vec![0x41],
                u64_constants: vec![0x42],
                ..Default::default()
            },
        );

        // get the same type we just register from the registry
        let ty = registry.resolve(tuple_type_id).unwrap().type_def.clone();

        // Generate a fuzzed encoded data for it
        let encoded = generator.generate(&mut fuzzer, &vec![ty]).unwrap();
        let parsing_pointer = &mut &encoded[..];

        // Decode it by hand
        assert_eq!(u32::decode(parsing_pointer).unwrap(), 0x41);
        assert_eq!(u64::decode(parsing_pointer).unwrap(), 0x42);

        assert!(u32::decode(parsing_pointer).is_err()); // No more elements
    }

    #[test]
    fn test_compact() {
        // Compact: A compact encoding of a value is encoded.

        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);

        // Compact
        let compact_type = Type::new(
            Path::default(),
            vec![],
            TypeDefCompact::new(u32_type_id.into()),
            vec![],
        );
        let compact_type_id = builder.register_type(compact_type);
        let registry = builder.finish();

        // build a generator for that registry
        let generator = Generator::new(&registry);
        // A constant fuzzer containing only 0x41 as u8 constants
        let mut fuzzer = Fuzzer::new(
            0,
            Constants {
                u32_constants: vec![0x41],
                ..Default::default()
            },
        );

        // get the same type we just register from the registry
        let ty = registry.resolve(compact_type_id).unwrap().type_def.clone();

        // Generate a fuzzed encoded data for it
        let encoded = generator.generate(&mut fuzzer, &vec![ty]).unwrap();
        let parsing_pointer = &mut &encoded[..];

        // Decode it by hand
        let value = <Compact<u32>>::decode(parsing_pointer).unwrap();
        assert_eq!(value.0, 0x41);
    }

    #[test]
    fn test_enum() {
        // Variant: A tagged union is encoded reseembling this: enum E{ VARIANTA(u32),
        // VARIANTB((u32,u32)) }
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);

        // Tuple
        let tuple_type = Type::new(
            Path::default(),
            vec![],
            TypeDefTuple::new_portable(vec![u32_type_id.into(), u32_type_id.into()]),
            vec![],
        );
        let tuple_type_id = builder.register_type(tuple_type);
        // Variant
        let enum_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyEnum".into()]))
            .variant(
                Variants::new()
                    .variant("VARIANTA".into(), |v| {
                        v.index(0).fields(
                            Fields::<PortableForm>::named()
                                .field_portable(|f| f.name("VA".into()).ty(u32_type_id)),
                        )
                    })
                    .variant("VARIANTB".into(), |v| {
                        v.index(1).fields(
                            Fields::<PortableForm>::named().field_portable(|f| {
                                f.name("VB".into()).ty(tuple_type_id)
                            }),
                        )
                    }),
            );
        let enum_type_id = builder.register_type(enum_type);
        let registry = builder.finish();

        // build a generator for that registry
        let generator = Generator::new(&registry);

        for rng_seed in 0..5 {
            // A constant fuzzer containing only 0x41 as u8 constants
            let mut fuzzer = Fuzzer::new(
                rng_seed,
                Constants {
                    u32_constants: vec![0x41],
                    ..Default::default()
                },
            );

            // get the same type we just register from the registry
            let ty = registry.resolve(enum_type_id).unwrap().type_def.clone();

            // Generate a fuzzed encoded data for it
            let encoded = generator.generate(&mut fuzzer, &vec![ty]).unwrap();
            let parsing_pointer = &mut &encoded[..];

            // Decode it by hand
            let variant = <u8>::decode(parsing_pointer).unwrap();
            match variant {
                0 => {
                    assert_eq!(u32::decode(parsing_pointer).unwrap(), 0x41);
                }
                1 => {
                    assert_eq!(u32::decode(parsing_pointer).unwrap(), 0x41);
                    assert_eq!(u32::decode(parsing_pointer).unwrap(), 0x41);
                }
                _ => {
                    panic!("Invalid variant index")
                }
            }
        }
    }
}
