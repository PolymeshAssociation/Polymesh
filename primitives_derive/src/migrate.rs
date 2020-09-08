use syn::export::TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{visit_mut, Data, DataEnum, DataStruct, DeriveInput, Ident, Index, Type};

/// Appends `Old` suffix to the `ident` returning the new appended `Ident`.
fn oldify_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("{}Old", ident), ident.span())
}

/// Appends `Old` suffix to those identifiers in `ty` as instructed by `refs`.
fn oldify_type(ty: &mut Type, refs: Option<MigrateRefs>) {
    let refs = match refs {
        None => return,
        Some(MigrateRefs::Any) => None,
        Some(MigrateRefs::Listed(list)) => Some(list),
        Some(MigrateRefs::Exact(old_ty)) => {
            *ty = old_ty;
            return;
        }
    };
    struct Vis(Option<Vec<Ident>>);
    impl visit_mut::VisitMut for Vis {
        fn visit_ident_mut(&mut self, ident: &mut Ident) {
            match &self.0 {
                // Got `#[migrate]`; instructed oldify any identifier.
                None => {}
                // Got `#[migrate(TypeA, TypeB, ...)]`, so check if `ident` is one of those.
                Some(list)
                    if {
                        let ident = ident.to_string();
                        list.iter().any(|elem| elem == &ident)
                    } => {}
                _ => return,
            }
            *ident = oldify_ident(ident);
        }
    }
    visit_mut::visit_type_mut(&mut Vis(refs), ty);
}

/// Go over the given `fields`,
/// stripping any `#[migrate]` attributes on them (while noting them), stripping those,
/// and then extending the fields in the output with the presence of `#[migrate]`.
fn fields_with_migration(fields: &mut syn::Fields) -> Vec<(bool, &syn::Field)> {
    let mut fields_vec = Vec::with_capacity(fields.len());
    for f in fields.iter_mut() {
        let refs = extract_migrate_refs(&mut f.attrs);
        let has_migrate = refs.is_some();
        oldify_type(&mut f.ty, refs);
        fields_vec.push((has_migrate, &*f));
    }
    fields_vec
}

/// Quote the given `fields`, assumed to be a variant/product,
/// into a pair of a destructuring (unpacking) pattern
/// and a piece of a struct/variant initialization expression.
fn quote_pack_unpack(fields: &[(bool, &syn::Field)]) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    fn pack(migrate: &bool, field: impl quote::ToTokens, var: &Ident) -> TokenStream2 {
        match migrate {
            true => quote!( #field: #var.migrate()? ),
            false => quote!( #field: #var ),
        }
    }
    fields
        .iter()
        .enumerate()
        .map(|(index, (migrate, field))| match &field.ident {
            Some(ident) => (quote!( #ident ), pack(migrate, &ident, &ident)),
            None => {
                let span = field.ty.span();
                let index = index as u32;
                let idx = Index { index, span };
                let var = Ident::new(&format!("idx{}", index), span);
                (quote!( #idx: #var ), pack(migrate, idx, &var))
            }
        })
        .unzip()
}

/// Implements `#[derive(Migrate)]`.
pub(crate) fn impl_migrate(mut input: DeriveInput) -> TokenStream2 {
    let name = input.ident;
    input.ident = oldify_ident(&name);
    let old_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let migration = match &mut input.data {
        Data::Union(_) => {
            return quote! {
                compile_error!("cannot derive `Migrate` for unions");
            }
        }
        Data::Struct(DataStruct { ref mut fields, .. }) => {
            // Handle `#[migrate]`s and old-ifying types
            // and then interpolate unpacking & packing.
            let (unpack, pack) = quote_pack_unpack(&fields_with_migration(fields));
            quote!( match self { Self { #(#unpack,)* } => Self::Into { #(#pack,)* } } )
        }
        Data::Enum(DataEnum {
            ref mut variants, ..
        }) => {
            // Same for each variant, as-if it were a struct.
            let arm = variants
                .iter_mut()
                .map(|syn::Variant { ident, fields, .. }| {
                    let (unpack, pack) = quote_pack_unpack(&fields_with_migration(fields));
                    quote!( Self::#ident { #(#unpack,)* } => Self::Into::#ident { #(#pack,)* } )
                });
            quote!( match self { #(#arm,)* } )
        }
    };

    quote! {
        #[derive(::codec::Decode)]
        #input

        impl #impl_generics polymesh_primitives::migrate::Migrate
        for #old_name #ty_generics
        #where_clause {
            type Into = #name #ty_generics;
            fn migrate(self) -> Option<Self::Into> { Some(#migration) }
        }
    }
}

/// Semantic representation of the `#[migrate]` attribute.
enum MigrateRefs {
    /// Derived from `#[migrate]`.
    /// Any identifier in the type of the field should be migrated.
    Any,
    /// Derived from `#[migrate(ident, ident, ...)]`.
    /// Only those identifiers in the list and which match in the type of the field should be migrated.
    Listed(Vec<Ident>),
    /// Do an exact replacement of the field type with the one given in `#[migrate_from(Type)]`.
    Exact(Type),
}

/// Returns information about any `#[migrate]` or `#[migrate_from]` attributes.
/// We also strip those attributes while at it.
///
/// The form `#[migrate = ".."]` does qualify.
fn extract_migrate_refs(attrs: &mut Vec<syn::Attribute>) -> Option<MigrateRefs> {
    let mut mig_ref = None;
    attrs.retain(|attr| {
        // Only care about `migrate{_from}`, and remove all of those, irrespective of form.
        let ident_str = attr.path.get_ident().map(|i| i.to_string());
        mig_ref = Some(match ident_str.as_deref() {
            // Got exactly `#[migrate]`.
            // User doesn't wish to specify which types to migrate, so assume all.
            Some("migrate") if attr.tokens.is_empty() => MigrateRefs::Any,
            // Got `#[migrate(ident, ident, ...)]` or maybe `#[migrate = "..."]`.
            Some("migrate") => {
                MigrateRefs::Listed(
                    attr.parse_args_with(|ps: ParseStream| {
                        // User only wants to oldify the given identifiers.
                        // Applies in e.g., `field: Vec<Foo>` where `Foo` is being migrated
                        // but `Vec` shouldn't be renamed as it is a container of `Foo`s.
                        ps.parse_terminated::<_, syn::Token![,]>(Ident::parse)
                            .map(|iter| iter.into_iter().collect())
                    })
                    .unwrap(),
                )
            }
            // Expect and parse `#[migrate_from($ty)]`.
            Some("migrate_from") => MigrateRefs::Exact(attr.parse_args().unwrap()),
            _ => return true,
        });
        false
    });
    mig_ref
}
