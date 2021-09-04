use proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(GenNodeIdent, attributes(flat_type))]
pub fn derive_node_ident(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let input_ident = &input.ident;
    let iter = input.variants.iter().map(map_variants);

    let idents = iter.clone().map(|MappedVariant { ident, .. }| ident);
    let from_match_arms = iter
        .clone()
        .map(|MappedVariant { from_match_arm, .. }| from_match_arm);
    let to_match_arms = iter
        .clone()
        .map(|MappedVariant { to_match_arm, .. }| to_match_arm);
    let res_match_arms = iter
        .clone()
        .filter_map(|MappedVariant { res_match_arm, .. }| res_match_arm);

    let result = quote! {
        pub enum NodeIdent {
            #(#idents),*
        }

        impl NodeIdent {
            fn from_str(s: &str) -> ::std::option::Option<NodeIdent> {
                match s {
                    #(#from_match_arms,)*
                    _ => ::std::option::Option::None,
                }
            }

            pub async fn resolve(self, uuid: &::uuid::Uuid, pool: &::sqlx::PgPool)
                -> ::std::option::Option<::std::option::Option<#input_ident>> {
                match self {
                    #(#res_match_arms,)*
                    _ => ::std::option::Option::None
                }
            }
        }

        impl ::std::fmt::Display for NodeIdent {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let str_representation = match self {
                    #(#to_match_arms,)*
                };
                f.write_str(str_representation)
            }
        }
    };

    result.into()
}

struct MappedVariant {
    ident: Ident,
    from_match_arm: proc_macro2::TokenStream,
    to_match_arm: proc_macro2::TokenStream,
    res_match_arm: Option<proc_macro2::TokenStream>,
}

fn map_variants(variant: &Variant) -> MappedVariant {
    let ident = &variant.ident;
    let meta_list = variant
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("flat_type"))
        .and_then(|attr| match attr.parse_meta() {
            Ok(syn::Meta::List(list)) => Some(list.nested),
            _ => None,
        });
    let str_lit = syn::LitStr::new(&ident.to_string(), ident.span());
    let mut resolver_fn_ident = None;

    if let Some(meta_list) = meta_list {
        for meta in meta_list {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(mnv)) = meta {
                if mnv.path.is_ident("resolver") {
                    if let Lit::Str(lit_str) = mnv.lit {
                        resolver_fn_ident = Some(Ident::new(&lit_str.value(), lit_str.span()));
                    }
                }
            }
        }
    }

    MappedVariant {
        ident: ident.clone(),
        from_match_arm: quote! {#str_lit => ::std::option::Option::Some(NodeIdent::#ident)},
        to_match_arm: quote! {NodeIdent::#ident => #str_lit},
        res_match_arm: resolver_fn_ident.map(|resolver_fn_ident| {
            quote! {
                NodeIdent::#ident => ::std::option::Option::Some(#resolver_fn_ident(uuid, pool).await)
            }
        }),
    }
}
