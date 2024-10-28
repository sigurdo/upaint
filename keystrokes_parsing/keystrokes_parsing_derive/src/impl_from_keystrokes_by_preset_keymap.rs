use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma,
    AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, GenericArgument, Path, PathArguments, PathSegment, Token, Type, TypePath,
    Variant,
};

use crate::utils::{ident_crate, join_by_comma, join_by_space};

fn presetify_fields(fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let ident_crate = ident_crate();
    fields
        .iter()
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let value = quote! {
                ::#ident_crate::keystrokes::Preset<#ty>
            };
            if let Some(ident) = ident {
                quote! {
                    #[serde(default)]
                    #ident: #value,
                }
            } else {
                quote! {
                    #value,
                }
            }
        })
        .reduce(join_by_space)
        .unwrap()
}

fn presetify_ident_fields(ident: &Ident, fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => {
            quote! {
                #ident
            }
        }
        Fields::Unnamed(unnamed) => {
            let fields = presetify_fields(&unnamed.unnamed);
            quote! {
               #ident(#fields)
            }
        }
        Fields::Named(named) => {
            let fields = presetify_fields(&named.named);
            quote! {
                #ident { #fields }
            }
        }
    }
}

fn from_preset(
    ident: &Ident,
    ident_preset: &Ident,
    variants: &Variant,
) -> proc_macro2::TokenStream {
    // let match_arms = fields.iter();
    quote! {
        impl FromPreset<#ident_preset> for #ident {
            fn from_preset(
                preset: #ident_preset,
                keystrokes: &mut crate::keystrokes::KeystrokeIterator,
                config: &crate::config::Config,
            ) -> Result<Self, KeybindCompletionError> {

                match preset {
                    crate::keystrokes::Preset::Preset(value) => {
                    },
                    crate::keystrokes::Preset::FromKeystrokes => {
                        Self::from_keystrokes(keystrokes, config)
                    },
                }
            }

        }
    }
}

struct Input {
    ident_preset: Ident,
    arrow: Token![=>],
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_preset = input.parse()?;
        let arrow = input.parse()?;
        let ident = input.parse()?;
        Ok(Input {
            ident_preset,
            arrow,
            ident,
        })
    }
}

pub fn impl_from_keystrokes_by_preset_keymap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let Input {
        ident_preset,
        arrow,
        ident,
    } = input;
    let ident_crate = ident_crate();
    let output = quote! {
        impl ::#ident_crate::FromKeystrokes<(), Config> for #ident {
            fn from_keystrokes(preset: (), keystrokes: &mut ::#ident_crate::KeystrokeIterator, config: &Config) -> Result<#ident, ::#ident_crate::FromKeystrokesError> {
                ::#ident_crate::from_keystrokes_by_preset_keymap(#ident_preset::get_keymap(config), keystrokes, config)
            }
        }
        impl ::#ident_crate::FromKeystrokes<#ident, Config> for #ident {
            fn from_keystrokes(preset: #ident, keystrokes: &mut ::#ident_crate::KeystrokeIterator, config: &Config) -> Result<#ident, ::#ident_crate::FromKeystrokesError> {
                Ok(preset)
            }
        }
    };
    let result = TokenStream::from(output);
    result
}
