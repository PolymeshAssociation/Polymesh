extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

/// Parses the AST to generate the code associated.
#[proc_macro_derive(VecU8StrongTyped)]
pub fn vec_u8_strong_typed_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_vec_u8_strong_typed(&ast)
}

/// Implements utility method for *strong typed* based on `Vec<u8>`.
fn impl_vec_u8_strong_typed(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl #name {

            /// Returns the number of elements.
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Extracts a slice containing the entire vector.
            #[inline]
            pub fn as_slice(&self) -> &[u8] {
                &self.0
            }

            /// Extracts the internal vector.
            #[inline]
            pub fn as_vec(&self) -> &Vec<u8> {
                &self.0
            }
        }

        impl<T: AsRef<[u8]>> From<T> for #name {
            #[inline]
            fn from(r: T) -> Self {
                #name(r.as_ref().to_vec())
            }
        }

        impl sp_std::ops::Deref for #name {
            type Target = [u8];

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.0.deref()
            }
        }

        impl sp_std::ops::DerefMut for #name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.0.deref_mut()
            }
        }
    };

    gen.into()
}

/// Parses the AST to generate the code associated.
#[proc_macro_derive(SliceU8StrongTyped)]
pub fn slice_u8_strong_typed_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_slice_u8_strong_typed(&ast)
}

/// Implements all utility method for *strong typed* based on `[u8]`.
fn impl_slice_u8_strong_typed(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl #name {
            /// Returns the number of elements.
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Extracts a slice containing the entire content.
            #[inline]
            pub fn as_slice(&self) -> &[u8] {
                &self.0[..]
            }
        }

        impl<T: AsRef<[u8]>> From<T> for #name {
            fn from(s: T) -> Self {
                let s = s.as_ref();
                let mut v = #name::default();
                let limit = sp_std::cmp::min(v.len(), s.len());

                let inner = &mut v.0[..limit];
                inner.copy_from_slice( &s[..limit]);

                v
            }
        }

        impl sp_std::ops::Deref for #name {
            type Target = [u8];

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0[..]
            }
        }

        impl sp_std::ops::DerefMut for #name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0[..]
            }
        }
    };

    gen.into()
}
