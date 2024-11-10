use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field,
    Fields, FieldsNamed, FieldsUnnamed, Type, Variant,
};

use crate::utils::{ident_crate, join_by_comma, join_by_space};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(presetable))]
struct Opts {
    preset_type: Option<String>,
    config_type: Option<String>,
    fields_required: bool,
}

#[derive(FromField, Default)]
#[darling(default, attributes(presetable))]
struct FieldOpts {
    required: bool,
    default: Option<String>,
}
impl FieldOpts {
    fn from_field_expect(field: &Field) -> Self {
        Self::from_field(field)
            .expect(format!("Wrong options for field {:?}", field.ident).as_str())
    }
}

#[derive(FromVariant, Default)]
#[darling(default, attributes(presetable))]
struct VariantOpts {
    required: bool,
}

fn presetify_fields(
    fields: &Punctuated<Field, Comma>,
    fields_required: bool,
) -> proc_macro2::TokenStream {
    let ident_crate = ident_crate();
    let result = fields
        .iter()
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let opts = FieldOpts::from_field_expect(field);
            let required = fields_required || opts.required;
            let value = if required {
                quote! {
                    <#ty as Presetable<Config>>::Preset
                }
            } else {
                quote! {
                    ::#ident_crate::PresetStructField<<#ty as Presetable<Config>>::Preset>
                }
            };
            if let Some(ident) = ident {
                if let Some(default) = opts.default {
                    quote! {
                        #[serde(default = #default)]
                        #ident: #value,
                    }
                } else {
                    quote! {
                        #[serde(default)]
                        #ident: #value,
                    }
                }
            } else {
                quote! {
                    #value,
                }
            }
        })
        .reduce(join_by_space);
    quote! { #result }
}

fn presetify_ident_fields(
    ident: &Ident,
    fields: &Fields,
    fields_required: bool,
) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => {
            quote! {
                #ident
            }
        }
        Fields::Unnamed(unnamed) => {
            let fields = presetify_fields(&unnamed.unnamed, fields_required);
            quote! {
               #ident(#fields)
            }
        }
        Fields::Named(named) => {
            let fields = presetify_fields(&named.named, fields_required);
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

pub fn derive_presetable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput {
        ident, data, vis, ..
    } = input;
    // panic!("required: {:#?}", opts.fields_required);
    let ident_preset = Ident::new(
        if let Some(preset_type) = opts.preset_type {
            preset_type
        } else {
            format!("{ident}Preset")
        }
        .as_str(),
        Span::call_site(),
    );
    let ident_config = Ident::new(
        if let Some(config_type) = opts.config_type {
            config_type
        } else {
            format!("Config")
        }
        .as_str(),
        Span::call_site(),
    );
    let ident_crate = ident_crate();
    let output = match data {
        Data::Enum(data) => {
            let variants = data
                .variants
                .iter()
                .map(|variant| {
                    let Variant { ident, fields, .. } = variant;
                    presetify_ident_fields(ident, fields, opts.fields_required)
                })
                .reduce(join_by_comma);

            let definition_enum = quote! {
                #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                #vis enum #ident_preset {
                    #variants
                }
            };
            let match_arms = data
                .variants
                .iter()
                .map(|variant| {
                    let Variant {
                        ident: ident_variant,
                        fields,
                        ..
                    } = variant;
                    match fields {
                        Fields::Unit => quote! {
                            #ident_preset::#ident_variant => Ok(#ident::#ident_variant)
                        },
                        Fields::Unnamed(FieldsUnnamed {
                            paren_token,
                            unnamed,
                        }) => {
                            let ident_args: Vec<Ident> = unnamed
                                .iter()
                                .enumerate()
                                .map(|(index, field)| {
                                    Ident::new(format!("arg_{index}").as_str(), Span::call_site())
                                })
                                .collect();
                            let arglist = ident_args
                                .iter()
                                .map(|ident| quote! { #ident })
                                .reduce(join_by_comma).unwrap();
                            let resultlist = unnamed
                                .iter()
                                .enumerate()
                                .map(|(index, field)| {
                                    let Field { ty, .. } = field;
                                    let ident_arg = Ident::new(
                                        format!("arg_{index}").as_str(),
                                        Span::call_site(),
                                    );
                                    let required = opts.fields_required || FieldOpts::from_field_expect(field).required;
                                    if required {
                                        quote! { #ty::from_keystrokes_by_preset(#ident_arg, keystrokes, config)? }
                                    } else {
                                        quote! { ::#ident_crate::from_keystrokes_by_preset_struct_field(#ident_arg, keystrokes, config)? }
                                    }
                                })
                                .reduce(join_by_comma).unwrap();
                            println!("arglist: {arglist}");
                            println!("resultlist: {resultlist}");
                            let dings = quote! {
                                #ident_preset::#ident_variant(#arglist) => Ok(#ident::#ident_variant(#resultlist))
                            };
                            println!("dings: {dings}");
                            dings
                        }
                        _ => panic!("Enum variants with named fields are not supported"),
                    }
                })
                .reduce(join_by_comma).unwrap();
            println!("match_arms: {match_arms}");

            let impl_presetable = quote! {
                impl Presetable<#ident_config> for #ident {
                    type Preset = #ident_preset;
                    fn from_keystrokes_by_preset(
                        preset: #ident_preset,
                        keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                        config: &#ident_config,
                    ) -> Result<Self, ::#ident_crate::FromKeystrokesError> {
                        match preset {
                            #match_arms
                        }
                    }

                }
                // Covered by generic impl in lib.rs
                // impl FromKeystrokes<::#ident_crate::Preset<#ident_preset>, #ident_config> for #ident {
                //     fn from_keystrokes(
                //         preset: #ident_preset,
                //         keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                //         config: #ident_config,
                //     ) -> Result<Self, KeybindCompletionError> {
                //         #ident::from_keystrokes(#ident_preset::from_preset(preset, keystrokes, config)?, keystrokes, config)
                //     }
                // }
                // impl FromKeystrokes<::#ident_crate::Preset<#ident_preset>> for #ident {
                //     fn from_keystrokes(
                //         preset: #ident_preset,
                //         keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                //         config: #ident_config,
                //     ) -> Result<Self, KeybindCompletionError> {
                //         let preset : crate::keystrokes::Preset<#ident_preset> = crate::keystrokes::Preset::FromKeystrokes;
                //         #ident::from_preset(preset, keystrokes, config)
                //     }
                // }
            };
            quote! {
                #definition_enum
                #impl_presetable
            }
        }
        Data::Struct(data) => {
            let DataStruct { fields, .. } = data;
            let fields_presetified =
                presetify_ident_fields(&ident_preset, &fields, opts.fields_required);
            let fields = match fields {
                Fields::Named(FieldsNamed { brace_token, named }) => named
                    .iter()
                    .map(|field| {
                        let Field { ty, ident, .. } = field;
                        let ident = ident.as_ref().expect("her er det noe");
                        let required = opts.fields_required || FieldOpts::from_field_expect(field).required;
                        if required {
                            quote! {
                                    #ident: #ty::from_keystrokes_by_preset(preset.#ident, keystrokes, config)?
                            }
                        } else {
                            quote! {
                                    #ident: ::#ident_crate::from_keystrokes_by_preset_struct_field(preset.#ident, keystrokes, config)?
                            }
                        }
                    })
                    .reduce(join_by_comma),
                _ => panic!("Struct with unnamed or unit fields are not supported."),
            };
            // panic!("Kom hit");
            let impl_from_preset = quote! {
                impl ::#ident_crate::Presetable<#ident_config> for #ident {
                    type Preset = #ident_preset;
                    fn from_keystrokes_by_preset(
                        preset: #ident_preset,
                        keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                        config: &#ident_config,
                    ) -> Result<Self, ::#ident_crate::FromKeystrokesError> {
                        Ok(Self {
                            #fields
                        })
                    }
                }
            };
            let result = quote! {
                #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                #vis struct #fields_presetified
                #impl_from_preset
            };
            result
        }
        _ => panic!(),
    };
    let result = TokenStream::from(output);
    println!("result derive Presetable: {result}");
    result
}
