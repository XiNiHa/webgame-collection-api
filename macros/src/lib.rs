use proc_macro::TokenStream;

mod error;
mod node_ident;

#[proc_macro_derive(Error, attributes(error))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    error::derive_error_impl(input)
}

#[proc_macro_derive(GenNodeIdent, attributes(node_ident))]
pub fn derive_node_ident(input: TokenStream) -> TokenStream {
    node_ident::node_ident_impl(input)
}
