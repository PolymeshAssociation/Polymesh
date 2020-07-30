extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod deserialize_u8_strong_typed;
mod serialize_u8_strong_typed;
mod slice_u8_strong_typed;
mod vec_u8_strong_typed;

use crate::{
    deserialize_u8_strong_typed::impl_deserialize_u8_strong_typed,
    serialize_u8_strong_typed::impl_serialize_u8_strong_typed,
    slice_u8_strong_typed::impl_slice_u8_strong_typed,
    vec_u8_strong_typed::impl_vec_u8_strong_typed,
};

/// Implements all utility method for *strong typed* based on `Vec<u8>` inner type.
#[proc_macro_derive(VecU8StrongTyped)]
pub fn vec_u8_strong_typed_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_vec_u8_strong_typed(&ast)
}

/// Implements all utility method for *strong typed* based on `[u8]` inner type.
#[proc_macro_derive(SliceU8StrongTyped)]
pub fn slice_u8_strong_typed_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_slice_u8_strong_typed(&ast)
}

/// Implements Serialize for `U8` strong typed types.
#[proc_macro_derive(SerializeU8StrongTyped)]
pub fn serialize_u8_strong_typed_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_serialize_u8_strong_typed(&ast)
}

/// Implements Deserialize for `U8` strong typed types.
#[proc_macro_derive(DeserializeU8StrongTyped)]
pub fn deserialize_u8_strong_typed_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_deserialize_u8_strong_typed(&ast)
}
