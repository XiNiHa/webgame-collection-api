use proc_macro::TokenStream;
use quote::quote;
use syn::*;

pub fn derive_error_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let input_ident = &input.ident;
    let iter = input.variants.iter().map(map_variants);

    if let Some(e) = iter.clone().find_map(|variant| variant.err()) {
        return e.to_compile_error().into();
    }

    let message_match_arms = iter.clone().map(std::result::Result::unwrap).map(
        |MappedVariant {
             ident,
             message,
             field_count,
         }| {
            let field_placeholders = get_field_placeholders(field_count);
            quote! { #input_ident::#ident#field_placeholders => #message }
        },
    );
    let code_match_arms = iter.clone().map(std::result::Result::unwrap).map(
        |MappedVariant {
             ident, field_count, ..
         }| {
            let lit = LitStr::new(&ident.to_string(), ident.span());
            let field_placeholders = get_field_placeholders(field_count);
            quote! { #input_ident::#ident#field_placeholders => #lit }
        },
    );
    let format_str = format!("{}::{{}}", input_ident);

    let result = quote! {
        impl crate::error::Error for #input_ident {
            fn message(&self) -> ::std::string::String {
                match self {
                    #(#message_match_arms,)*
                }
                .to_owned()
            }

            fn code(&self) -> String {
                let self_in_str = match self {
                    #(#code_match_arms,)*
                };

                format!(#format_str, self_in_str)
            }
        }
    };

    result.into()
}

struct MappedVariant {
    ident: Ident,
    message: LitStr,
    field_count: usize,
}

fn map_variants(variant: &Variant) -> std::result::Result<MappedVariant, Error> {
    let ident = &variant.ident;
    let message = variant
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("error"))
        .and_then(|attr| match attr.parse_meta() {
            Ok(Meta::List(list)) => list.nested.iter().find_map(|meta| match meta {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(lit_str),
                    ..
                })) => match path.is_ident("message") {
                    true => Some(lit_str.clone()),
                    false => None,
                },
                _ => None,
            }),
            _ => None,
        })
        .ok_or(Error::new_spanned(
            ident,
            "`#[error(message = \"...\")]` is mandatory",
        ))?;
    let field_count = variant.fields.len();

    Ok(MappedVariant {
        ident: ident.clone(),
        message: message.clone(),
        field_count,
    })
}

fn get_field_placeholders(field_count: usize) -> Option<proc_macro2::TokenStream> {
    if field_count > 0 {
        let placeholders = vec![quote! {_}; field_count];
        Some(quote! {(#(#placeholders),*)})
    } else {
        None
    }
}
