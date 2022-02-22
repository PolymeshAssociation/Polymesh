use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::iter::FromIterator;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, Token,
};

#[derive(Debug)]
struct Exec {
    pallet: Ident,
    extrinsic: Ident,
    origin: Expr,
    params: Punctuated<Expr, Token![,]>,
}

impl Parse for Exec {
    fn parse(input: ParseStream) -> Result<Self> {
        let pallet: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let extrinsic: Ident = input.parse()?;

        let paren_content;
        parenthesized!(paren_content in input);
        let mut params = paren_content
            .parse_terminated::<Expr, Token![,]>(Expr::parse)?
            .into_iter();
        let origin = params.next().unwrap();
        let params = Punctuated::from_iter(params);

        Ok(Exec {
            pallet,
            extrinsic,
            origin,
            params,
        })
    }
}

/// Convert a direct call to a module to d extrinsic execution.
/// When `#[integration-test]` is not set, this becomes a noop.
/// Syntax: `<pallet>::<extrinsic>(<params>,*)`
#[proc_macro]
pub fn exec(item: TokenStream) -> TokenStream {
    if !cfg!(feature = "integration-test") {
        return item;
    }

    let Exec {
        pallet,
        extrinsic,
        origin,
        params,
    } = parse_macro_input!(item as Exec);

    let call_variant = Ident::new(
        &format!("new_call_variant_{}", extrinsic.to_string()),
        Span::call_site(),
    );

    let token_stream = quote! {
        crate::storage::exec(
            #origin,
            <crate::storage::#pallet as frame_support::dispatch::Callable<crate::TestStorage>>::Call::#call_variant(
                #params
            )
        )
    };

    TokenStream::from(token_stream)
}
