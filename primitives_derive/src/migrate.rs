use syn::export::TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    visit_mut, Data, DataEnum, DataStruct, DeriveInput, Ident, Index, Type,
};

enum Field {
    Named(Ident),
    Index(usize),
}

impl Field {
    fn new(idx: usize, opt_ident: Option<Ident>) -> Self {
        match opt_ident {
            Some(ident) => Self::Named(ident),
            None => Self::Index(idx),
        }
    }

    fn idx_ident(idx: usize) -> (Ident, Index) {
        let idx = Index::from(idx);
        let ident = Ident::new(&format!("idx{}", idx.index), idx.span);
        (ident, idx)
    }

    fn pat(&self) -> TokenStream2 {
        match self {
            Field::Named(ident) => quote!( #ident ),
            Field::Index(idx) => {
                let (ident, idx) = Self::idx_ident(*idx);
                quote!( #idx: #ident )
            }
        }
    }

    fn init(&self, migrate: bool) -> TokenStream2 {
        match (self, migrate) {
            (Field::Named(ident), true) => quote!( #ident: #ident.migrate()? ),
            (Field::Named(ident), _) => quote!( #ident ),
            (Field::Index(idx), true) => {
                let (var, idx) = Self::idx_ident(*idx);
                quote!( #idx: #var.migrate()? )
            }
            (Field::Index(idx), false) => {
                let (var, idx) = Self::idx_ident(*idx);
                quote!( #idx: #var )
            }
        }
    }
}

fn oldify_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("{}Old", ident), ident.span())
}

fn oldify_type(ty: &mut Type, refs: &Option<MigrateRefs>) {
    let refs = match refs {
        None => return,
        Some(refs) => refs,
    };
    struct Vis<'a>(&'a MigrateRefs);
    impl visit_mut::VisitMut for Vis<'_> {
        fn visit_ident_mut(&mut self, ident: &mut Ident) {
            match &self.0 {
                MigrateRefs::Any => {}
                MigrateRefs::Listed(list)
                    if {
                        let ident = ident.to_string();
                        list.iter().any(|elem| elem == &ident)
                    } => {}
                _ => return,
            }
            *ident = oldify_ident(ident);
        }
    }
    visit_mut::visit_type_mut(&mut Vis(&refs), ty);
}

type FieldsVec<'a> = Vec<(Option<MigrateRefs>, &'a mut syn::Field)>;

fn fields_vec(fields: &mut syn::Fields) -> FieldsVec<'_> {
    let mut fields_vec = Vec::with_capacity(fields.len());
    for f in fields.iter_mut() {
        let refs = has_migrate(&mut f.attrs);
        oldify_type(&mut f.ty, &refs);
        fields_vec.push((refs, f));
    }
    fields_vec
}

fn pack_unpack(fields_vec: &FieldsVec<'_>) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    fields_vec
        .iter()
        .enumerate()
        .map(|(idx, (refs, field))| {
            let field = Field::new(idx, field.ident.clone());
            (field.pat(), field.init(refs.is_some()))
        })
        .unzip()
}

/// Implements all utility method for *strong typed* based on `[u8]`.
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
            let (unpack, pack) = pack_unpack(&fields_vec(fields));
            quote!( match self { Self { #(#unpack,)* } => Self::Into { #(#pack,)* } } )
        }
        Data::Enum(DataEnum {
            ref mut variants, ..
        }) => {
            // Same for each variant, as-if it were a struct.
            let arm = variants
                .iter_mut()
                .map(|syn::Variant { ident, fields, .. }| {
                    let (unpack, pack) = pack_unpack(&fields_vec(fields));
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

enum MigrateRefs {
    Any,
    Listed(Vec<syn::Ident>),
}

/// Returns whether `attrs` contains `#[migrate]` and also strips such attributes.
///
/// Forms `#[migrate(...)]` nor `#[migrate = ".."]` do not qualify.
fn has_migrate(attrs: &mut Vec<syn::Attribute>) -> Option<MigrateRefs> {
    let mut mig_ref = None;
    attrs.retain(|attr| {
        if attr.path.is_ident("migrate") {
            if attr.tokens.is_empty() {
                mig_ref = Some(MigrateRefs::Any);
            } else if let Ok(refs) = attr.parse_args_with(|ps: ParseStream| {
                ps.parse_terminated::<_, syn::Token![,]>(Ident::parse)
                    .map(|iter| iter.into_iter().collect())
            }) {
                mig_ref = Some(MigrateRefs::Listed(refs));
            }
            false
        } else {
            true
        }
    });
    mig_ref
}
