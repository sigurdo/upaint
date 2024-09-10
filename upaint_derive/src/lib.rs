use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field,
    Fields, FieldsUnnamed, Variant,
};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(preset))]
struct Opts {
    preset_type: Option<String>,
}

fn join_by_space(
    a: proc_macro2::TokenStream,
    b: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #a #b
    }
}
fn join_by_comma(
    a: proc_macro2::TokenStream,
    b: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        #a, #b
    }
}

fn presetify_fields(fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    fields
        .iter()
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let value = quote! {
                crate::keystrokes::Preset<#ty>
            };
            if let Some(ident) = ident {
                quote! {
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

#[proc_macro_derive(Preset, attributes(preset))]
pub fn derive_preset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput {
        ident, data, vis, ..
    } = input;
    let ident_preset = Ident::new(
        if let Some(preset_type) = opts.preset_type {
            preset_type
        } else {
            format!("{ident}Preset")
        }
        .as_str(),
        Span::call_site(),
    );
    let output = match data {
        Data::Enum(data) => {
            let variants = data
                .variants
                .iter()
                .map(|variant| {
                    let Variant { ident, fields, .. } = variant;
                    presetify_ident_fields(ident, fields)
                })
                .reduce(join_by_comma);

            let definition_enum = quote! {
                #[derive(::core::fmt::Debug, ::core::clone::Clone, ::serde::Serialize, ::serde::Deserialize)]
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
                                    quote! { #ty::from_preset(#ident_arg, keystrokes, config)? }
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
                        _ => panic!("Not supported"),
                    }
                })
                .reduce(join_by_comma).unwrap();
            println!("match_arms: {match_arms}");

            let impl_from_preset = quote! {
                impl FromPreset<#ident_preset> for #ident {
                    fn from_preset(
                        preset: #ident_preset,
                        keystrokes: &mut crate::keystrokes::KeystrokeIterator,
                        config: &crate::config::Config,
                    ) -> Result<Self, KeybindCompletionError> {
                        match preset {
                            #match_arms
                        }
                    }

                }
                impl FromPreset<crate::keystrokes::Preset<#ident_preset>> for #ident {
                    fn from_preset(
                        preset: crate::keystrokes::Preset<#ident_preset>,
                        keystrokes: &mut crate::keystrokes::KeystrokeIterator,
                        config: &crate::config::Config,
                    ) -> Result<Self, KeybindCompletionError> {
                        #ident::from_preset(#ident_preset::from_preset(preset, keystrokes, config)?, keystrokes, config)
                    }
                }
                impl FromKeystrokes for #ident {
                    fn from_keystrokes(
                        keystrokes: &mut crate::keystrokes::KeystrokeIterator,
                        config: &crate::config::Config,
                    ) -> Result<Self, KeybindCompletionError> {
                        let preset : crate::keystrokes::Preset<#ident_preset> = crate::keystrokes::Preset::FromKeystrokes;
                        #ident::from_preset(preset, keystrokes, config)
                    }
                }
            };
            quote! {
                #definition_enum
                #impl_from_preset
            }
        }
        Data::Struct(data) => {
            let DataStruct { fields, .. } = data;
            let fields_presetified = presetify_ident_fields(&ident_preset, &fields);
            quote! {
                #[derive(::core::fmt::Debug, ::core::clone::Clone, ::serde::Serialize, ::serde::Deserialize)]
                #vis struct #fields_presetified;
            }
        }
        _ => panic!(),
    };
    let result = TokenStream::from(output);
    result
}
