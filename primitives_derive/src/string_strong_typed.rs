use proc_macro::TokenStream;

/// Implements utility method for *strong typed* based on `String`.
pub(crate) fn impl_string_strong_typed(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl #name {

            /// Returns the number of elements.
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Returns `true` if there are no elements.
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            /// Extracts a slice containing the entire String.
            #[inline]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Converts to a byte slice.
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }

        impl<T: AsRef<str>> From<T> for #name {
            #[inline]
            fn from(r: T) -> Self {
                #name(r.as_ref().to_string())
            }
        }

        impl sp_std::ops::Deref for #name {
            type Target = str;

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
