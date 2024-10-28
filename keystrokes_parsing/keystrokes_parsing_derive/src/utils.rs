use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field,
    Fields, FieldsNamed, FieldsUnnamed, Variant,
};

pub fn join_by_space(
    a: proc_macro2::TokenStream,
    b: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #a #b
    }
}
pub fn join_by_comma(
    a: proc_macro2::TokenStream,
    b: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #a, #b
    }
}

pub fn ident_crate() -> Ident {
    // env!("CARGO_CRATE_NAME")
    Ident::new("keystrokes_parsing", Span::call_site())
}
