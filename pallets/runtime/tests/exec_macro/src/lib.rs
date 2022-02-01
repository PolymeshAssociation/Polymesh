use proc_macro::TokenStream;
use quote::quote;
use std::iter::FromIterator;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Error, Expr, Ident, Token,
};

#[derive(Debug)]
enum AssertType {
    Ok,
    NoOp(Expr),
    Err(Expr),
}

#[derive(Debug)]
struct Exec {
    pallet: Ident,
    extrinsic: Ident,
    origin: Expr,
    params: Punctuated<Expr, Token![,]>,
    error: AssertType,
}

impl Parse for Exec {
    fn parse(input: ParseStream) -> Result<Self> {
        let pallet: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let extrinsic: Ident = input.parse()?;

        let paren_content;
        parenthesized!(paren_content in input);
        let params = paren_content.parse_terminated::<Expr, Token![,]>(Expr::parse)?;
        let origin = params.iter().next().unwrap().clone();
        let params = Punctuated::from_iter(params.into_iter().skip(1));
        input.parse::<Token![.]>()?;
        let assert_method: Ident = input.parse()?;

        let paren_content;
        parenthesized!(paren_content in input);

        let error = match assert_method.to_string().as_str() {
            "ok" => AssertType::Ok,
            "noop" => AssertType::NoOp(paren_content.parse()?),
            "err" => AssertType::Err(paren_content.parse()?),
            _ => {
                return Err(Error::new(
                    paren_content.span(),
                    "Must be one of `ok`, `noop` or `err`",
                ))
            }
        };

        Ok(Exec {
            pallet,
            extrinsic,
            origin,
            params,
            error,
        })
    }
}

#[proc_macro]
pub fn exec(item: TokenStream) -> TokenStream {
    let Exec {
        pallet,
        extrinsic,
        origin,
        params,
        error,
    } = parse_macro_input!(item as Exec);

    let token_stream = match error {
        AssertType::Ok => {
            quote! {
                let result = crate::exec(
                    #origin,
                    <crate::storage::#pallet as frame_support::dispatch::Callable<crate::TestStorage>>::Call::#extrinsic(
                        #params
                    )
                );
                frame_support::assert_ok!(result);
            }
        }
        AssertType::NoOp(err) | AssertType::Err(err) => {
            quote! {
                let result = crate::exec(
                    #origin,
                    <crate::storage::#pallet as frame_support::dispatch::Callable<crate::TestStorage>>::Call::#extrinsic(
                        #params
                    )
                );
                frame_support::assert_err!(result, #err);
            }
        }
    };

    TokenStream::from(token_stream)
}
