use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field,
    Fields, FieldsNamed, FieldsUnnamed, Variant,
};

use crate::utils::{ident_crate, join_by_comma, join_by_space};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(presetable))]
struct Opts {
    // If set to None, "Preset" will be appended to the name of the type Presetable is
    // derived for.
    // Can be set to "Self" to not generate a new preset-type just a Presetable implementation.
    preset_type: Option<String>,
    config_type: Option<String>,
    all_required: bool,
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
    config_type: Ident,
    all_required: bool,
) -> proc_macro2::TokenStream {
    let ident_crate = ident_crate();
    let result = fields
        .iter()
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let opts = FieldOpts::from_field_expect(field);
            let required = all_required || opts.required;
            let value = if required {
                quote! {
                    <#ty as Presetable<#config_type>>::Preset
                }
            } else {
                quote! {
                    ::#ident_crate::PresetStructField<<#ty as Presetable<#config_type>>::Preset>
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
    config_type: Ident,
    all_required: bool,
) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => {
            quote! {
                #ident
            }
        }
        Fields::Unnamed(unnamed) => {
            let fields = presetify_fields(&unnamed.unnamed, config_type, all_required);
            quote! {
               #ident(#fields)
            }
        }
        Fields::Named(named) => {
            let fields = presetify_fields(&named.named, config_type, all_required);
            quote! {
                #ident { #fields }
            }
        }
    }
}

pub fn derive_presetable_by_self(derive_for: Ident, config: Ident) -> TokenStream {
    quote! {
        impl Presetable<#config> for #derive_for {
            type Preset = Self;
            fn from_keystrokes_by_preset(
                preset: Self::Preset,
                _keystrokes: &mut keystrokes_parsing::KeystrokeIterator,
                _config: &#config,
            ) -> Result<Self, keystrokes_parsing::FromKeystrokesError> {
                Ok(preset)
            }
        }
    }
    .into()
}

pub fn derive_presetable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput {
        ident, data, vis, ..
    } = input;
    let ident_config = Ident::new(
        if let Some(ref config_type) = opts.config_type {
            config_type.clone()
        } else {
            format!("Config")
        }
        .as_str(),
        Span::call_site(),
    );
    let ident_preset = Ident::new(
        if let Some(ref preset_type) = opts.preset_type {
            if preset_type == "Self" {
                return derive_presetable_by_self(ident, ident_config);
            }
            preset_type.clone()
        } else {
            format!("{ident}Preset")
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
                    presetify_ident_fields(ident, fields, ident_config.clone(), opts.all_required)
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
                            paren_token: _,
                            unnamed,
                        }) => {
                            let ident_args: Vec<Ident> = unnamed
                                .iter()
                                .enumerate()
                                .map(|(index, _field)| {
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
                                    let required = opts.all_required || FieldOpts::from_field_expect(field).required;
                                    if required {
                                        quote! { #ty::from_keystrokes_by_preset(#ident_arg, keystrokes, config)? }
                                    } else {
                                        quote! { ::#ident_crate::from_keystrokes_by_preset_struct_field(#ident_arg, keystrokes, config)? }
                                    }
                                })
                                .reduce(join_by_comma).unwrap();
                            let dings = quote! {
                                #ident_preset::#ident_variant(#arglist) => Ok(#ident::#ident_variant(#resultlist))
                            };
                            dings
                        }
                        _ => panic!("Enum variants with named fields are not supported"),
                    }
                })
                .reduce(join_by_comma).unwrap();
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
            };
            quote! {
                #definition_enum
                #impl_presetable
            }
        }
        Data::Struct(data) => {
            let DataStruct { fields, .. } = data;
            let fields_presetified =
                presetify_ident_fields(&ident_preset, &fields, ident_config.clone(), opts.all_required);
            fn create_fields(
                fields: &Punctuated<Field, Comma>,
                opts: &Opts,
            ) -> Option<proc_macro2::TokenStream> {
                fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let Field { ty, ident, .. } = field;
                        let (ident, ident_colon) = if let Some(ident) = ident.as_ref() {
                            (quote!{ #ident }, quote! { #ident: })
                        } else {
                            
                            let ident = syn::LitInt::new(format!("{index}").as_str(), Span::call_site());
                            (quote! { #ident }, quote! {})
                        };
                        let required = opts.all_required || FieldOpts::from_field_expect(field).required;
                        if required {
                            quote! {
                                #ident_colon #ty::from_keystrokes_by_preset(preset.#ident, keystrokes, config)?,
                            }
                        } else {
                            let ident_crate = crate::utils::ident_crate();
                            quote! {
                                #ident_colon ::#ident_crate::from_keystrokes_by_preset_struct_field(preset.#ident, keystrokes, config)?,
                            }
                        }
                    })
                    .reduce(join_by_space)
            }
            let from_keystrokes_instantiation = match fields {
                Fields::Named(FieldsNamed {
                    brace_token: _,
                    ref named,
                }) => {
                    let fields = create_fields(named, &opts);
                    quote! { Self { #fields } }
                }
                Fields::Unnamed(FieldsUnnamed {
                    paren_token: _,
                    ref unnamed,
                }) => {
                    let fields = create_fields(unnamed, &opts);
                    quote! { Self ( #fields ) }
                }

                _ => panic!("Struct with unnamed or unit fields are not supported."),
            };
            let impl_from_preset = quote! {
                impl ::#ident_crate::Presetable<#ident_config> for #ident {
                    type Preset = #ident_preset;
                    fn from_keystrokes_by_preset(
                        preset: #ident_preset,
                        keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                        config: &#ident_config,
                    ) -> Result<Self, ::#ident_crate::FromKeystrokesError> {
                        Ok( #from_keystrokes_instantiation )
                    }
                }
            };
            let impl_from_keystrokes_by_preset_keymap = quote! {
                impl ::#ident_crate::FromKeystrokesBy<&::#ident_crate::Keymap<#ident_preset>, #ident_config> for #ident {
                    fn from_keystrokes_by(
                        by: &#ident_crate::Keymap<#ident_preset>,
                        keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                        config: &#ident_config,
                    ) -> Result<Self, ::#ident_crate::FromKeystrokesError> {
                        ::#ident_crate::from_keystrokes_by_preset_keymap(by, keystrokes, config)
                    }
                }
            };
            let semicolon = if let Fields::Unnamed(_) = fields {
                quote! { ; }
            } else {
                quote! {}
            };
            let result = quote! {
                #[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::cmp::PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                #vis struct #fields_presetified #semicolon
                #impl_from_preset
                #impl_from_keystrokes_by_preset_keymap
            };
            result
        }
        _ => panic!(),
    };
    let result = TokenStream::from(output);
    // println!("result derive Presetable: {result}");
    result
}
