use crate::get_keymap::find_keymaps;
use crate::get_keymap::FindKeymapsResults;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, AngleBracketedGenericArguments, Data,
    DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Path,
    PathArguments, PathSegment, Type, TypePath, Variant,
};

use crate::utils::{ident_crate, join_by_comma, join_by_space};

fn presetify_fields(fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let ident_crate = ident_crate();
    fields
        .iter()
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let value = quote! {
                ::#ident_crate::keystrokes::PresetStructField<#ty>
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

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(from_keystrokes))]
struct Opts {
    config: Option<String>,
}
pub fn derive_from_keystrokes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput {
        ident, data, vis, ..
    } = input;
    let ident_config = if let Some(config) = opts.config {
        Ident::new(config.as_str(), Span::call_site())
    } else {
        ident
    };
    // let ident_config = opts.from_keystrokes_config;
    // panic!("hei config: {:#}", ident_config);
    let ident_crate = ident_crate();
    let output = match data {
        Data::Enum(data) => {
            panic!("Config enum not supported");
        }
        Data::Struct(data) => {
            let output = find_keymaps(&data)
                .map(
                    |FindKeymapsResults {
                         ident,
                         type_preset,
                         types_for,
                     }| {
                        let impl_from_keystrokes = types_for.iter().map(|type_for| {
                    quote! {
                        impl ::#ident_crate::FromKeystrokes<#ident_config> for #type_for {
                            fn from_keystrokes(
                                keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                                config: &#ident_config,
                            ) -> Result<#type_for, ::#ident_crate::FromKeystrokesError> {
                                ::#ident_crate::from_keystrokes_by_preset_keymap(
                                    config.get_keymap(),
                                    // #type_preset::get_keymap(config),
                                    keystrokes,
                                    config,
                                )
                            }
                        }
                    }
                }).reduce(join_by_space);
                        quote! {
                            #impl_from_keystrokes
                        }
                    },
                )
                .reduce(join_by_space);
            let output = quote! {#output};
            println!("output from_keystrokes: {output}");
            output
        }
        Data::Union(_) => panic!("Config union not supported"),
    };
    let result = TokenStream::from(output);
    result
}
