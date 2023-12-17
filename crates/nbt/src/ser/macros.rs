/// Creates a enum consisting of fieldless variants
/// corresponding to serde types which are invalid for the specific serializer.
macro_rules! unserializable_type {
    (
        $(#[$enum_meta:meta])*
        $enum_vis:vis enum $enum_name:ident {
            $($variant_name:ident $(as $stringified_variant:literal)?),*$(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(
            ::std::fmt::Debug,
            ::std::clone::Clone,
            ::std::marker::Copy,
            ::std::cmp::Eq,
            ::std::cmp::PartialEq
        )]
        $enum_vis enum $enum_name {
            $($variant_name,)*
        }

        impl ::std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(match self {$(
                    Self::$variant_name => $crate::ser::unserializable_type! {
                        @__stringify_variant $variant_name $(as $stringified_variant)?
                    },
                )*})
            }
        }
    };
    // explicit name mappings
    (@__stringify_variant $variant_name:ident as $stringified_variant:literal) => {
        $stringified_variant
    };
    // implicit name mappings
    (@__stringify_variant Bool) => { "bool" };
    (@__stringify_variant I8) => { "i8" };
    (@__stringify_variant U8) => { "u8" };
    (@__stringify_variant I16) => { "i16" };
    (@__stringify_variant U16) => { "u16" };
    (@__stringify_variant I32) => { "i32" };
    (@__stringify_variant U32) => { "u32" };
    (@__stringify_variant I64) => { "i64" };
    (@__stringify_variant U64) => { "u64" };
    (@__stringify_variant I128) => { "i128" };
    (@__stringify_variant U128) => { "u128" };
    (@__stringify_variant F32) => { "f32" };
    (@__stringify_variant F64) => { "f64" };
    (@__stringify_variant Char) => { "char" };
    (@__stringify_variant Str) => { "str" };
    (@__stringify_variant Bytes) => { "bytes" };
    (@__stringify_variant None) => { "none" };
    (@__stringify_variant Some) => { "some" };
    (@__stringify_variant EmptyTuple) => { "empty tuple" };
    (@__stringify_variant Unit) => { "unit" };
    (@__stringify_variant UnitStruct) => { "unit struct" };
    (@__stringify_variant UnitVariant) => { "unit variant" };
    (@__stringify_variant NewtypeStruct) => { "newtype struct" };
    (@__stringify_variant NewtypeVariant) => { "newtype variant" };
    (@__stringify_variant Seq) => { "seq" };
    (@__stringify_variant Tuple) => { "tuple" };
    (@__stringify_variant TupleStruct) => { "tuple struct" };
    (@__stringify_variant TupleVariant) => { "tuple variant" };
    (@__stringify_variant Map) => { "map" };
    (@__stringify_variant Struct) => { "struct" };
    (@__stringify_variant StructVariant) => { "struct variant" };
    (@__stringify_variant StructVariant as ) => { "struct variant" };
}

/// Re-export required by the macro internals.
pub(crate) use unserializable_type;
