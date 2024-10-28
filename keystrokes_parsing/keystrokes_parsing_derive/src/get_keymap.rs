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

pub struct FindKeymapsResults<'a> {
    ident: &'a Ident,
    type_preset: &'a Type,
    types_for: Vec<Type>,
}
pub fn find_keymaps<'a>(data: &'a DataStruct) -> impl Iterator<Item = FindKeymapsResults<'a>> {
    fn inner_type_of_keymaps<'a>(ty: &'a Type) -> Result<&'a Type, ()> {
        match ty {
            Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            }) => {
                let ident_keymaps = Ident::new("Keymap", Span::call_site());
                if let Some(PathSegment { ident, arguments }) = segments.get(0) {
                    if *ident == ident_keymaps {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) = arguments
                        {
                            if args.len() != 1 {
                                panic!("Found Keymap with more than one type arguments: {args:#?}");
                            }
                            match args.get(0) {
                                Some(GenericArgument::Type(type_inner)) => return Ok(type_inner),
                                _ => (),
                            }
                        }
                    }
                }
            }
            _ => (),
        };
        Err(())
    }
    let DataStruct { fields, .. } = data;
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => named.iter().filter_map(|field| {
            let Field {
                ty, ident, attrs, ..
            } = field;
            let ident = ident.as_ref().expect("Found named field without ident");
            let Ok(type_preset) = inner_type_of_keymaps(ty) else {
                return None;
            };
            let type_for = ty;
            let types_for = attrs
                .iter()
                .filter_map(|attr| {
                    if attr.path().is_ident("preset_for") {
                        let ty_for: Type = attr
                            .parse_args()
                            .expect("Cannot parse argument of preset_for as type");
                        Some(ty_for)
                    } else {
                        None
                    }
                })
                .collect();
            Some(FindKeymapsResults {
                ident,
                type_preset,
                types_for,
            })
        }),
        _ => panic!("Struct with unnamed or unit fields are not supported."),
    }
}

pub fn derive_get_keymap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput {
        ident: ident_config,
        data,
        vis,
        ..
    } = input;
    let ident_crate = ident_crate();
    let output = match data {
        Data::Enum(data) => {
            panic!("Config enum not supported");
        }
        Data::Struct(data) => {
            // let mut impl_from_keystrokes = proc_macro2::TokenStream::new();
            let output = find_keymaps(&data).map(|FindKeymapsResults { ident, type_preset, types_for }| {
                let impl_from_keystrokes = types_for.iter().map(|type_for| {
                    quote! {
                        impl ::#ident_crate::FromKeystrokes<Config> for #type_for {
                            fn from_keystrokes(
                                keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                                config: &Config,
                            ) -> Result<#type_for, ::#ident_crate::FromKeystrokesError> {
                                ::#ident_crate::from_keystrokes_by_preset_keymap(
                                    #type_preset::get_keymap(config),
                                    keystrokes,
                                    config,
                                )
                            }
                        }

                        // impl ::#ident_crate::FromKeystrokes<::#ident_crate::PresetStructField<#type_preset>, Config> for #type_for {
                        //     fn from_keystrokes(
                        //         preset: ::#ident_crate::PresetStructField<#type_preset>,
                        //         keystrokes: &mut ::#ident_crate::KeystrokeIterator,
                        //         config: &Config,
                        //     ) -> Result<#type_for, ::#ident_crate::FromKeystrokesError> {
                        //         ::#ident_crate::from_keystrokes_by_preset(
                        //             preset,
                        //             keystrokes,
                        //             config,
                        //         )
                        //     }
                        // }
                    }
                }).reduce(join_by_space);
                quote! {
                    #impl_from_keystrokes
                    impl GetKeymap<#ident_config> for #type_preset {
                        fn get_keymap<'a>(config: &'a #ident_config) -> &'a ::#ident_crate::Keymap<#type_preset> {
                            &config.#ident
                        }
                    }
                }
            })
            .reduce(join_by_space);
            let output = quote! {#output};

            // let DataStruct { fields, .. } = data;
            // let mut impl_from_keystrokes = proc_macro2::TokenStream::new();
            // let output = match fields {
            //     Fields::Named(FieldsNamed { brace_token, named }) => named
            //         .iter()
            //         .map(|field| {
            //             let Field { ty, ident, attrs, .. } = field;
            //             fn inner_type_of_keymaps<'a>(ty: &'a Type) -> Option<&'a GenericArgument> {
            //                 match ty {
            //                     Type::Path(TypePath {
            //                         path:
            //                             Path {
            //                                 segments,
            //                                 ..
            //                             },
            //                         ..
            //                     }) => {
            //                         let ident_keymaps = Ident::new("Keymap", Span::call_site());
            //                         if let Some(PathSegment { ident, arguments }) = segments.get(0) {
            //                             if *ident == ident_keymaps {
            //                                 if let PathArguments::AngleBracketed(
            //                                     AngleBracketedGenericArguments { args, .. },
            //                                 ) = arguments
            //                                 {
            //                                     if args.len() != 1 {
            //                                         panic!("Found Keymap with more than one type arguments: {args:#?}");
            //                                     }
            //                                     return Some(args.get(0).unwrap())
            //                                     // if let Some(GenericArgument::Type(Type::Path(
            //                                     //     TypePath {
            //                                     //         qself,
            //                                     //         path:
            //                                     //             Path {
            //                                     //                 leading_colon,
            //                                     //                 segments,
            //                                     //             },
            //                                     //     },
            //                                     // ))) = args.get(0)
            //                                     // {
            //                                     // }
            //                                     // if let Some()
            //                                 }
            //                             }
            //                         }
            //                     }
            //                     _ => ()
            //                 };
            //                 None
            //             }
            //             let ty_inner = inner_type_of_keymaps(ty);
            //             let impl_from_keystrokes_append = attrs.iter().filter_map(|attr| {
            //                 if attr.path().is_ident("preset_for") {
            //                     let ty_for: Type = attr.parse_args().expect("Cannot parse argument of preset_for as type");
            //                     Some(ty_for)
            //                 } else {
            //                     None
            //                 }
            //             }).map(|ty_for| {
            //                 quote! {
            //                     // impl ::#ident_crate::FromKeystrokes<#ty_for, Config> for #ty_for {
            //                     //     fn from_keystrokes(preset: #ty_for, keystrokes: &mut ::#ident_crate::KeystrokeIterator, config: &Config) -> Result<#ty_for, ::#ident_crate::FromKeystrokesError> {
            //                     //         Ok(preset)
            //                     //     }
            //                     // }
            //                     impl ::#ident_crate::FromKeystrokes<(), Config> for #ty_for {
            //                         fn from_keystrokes(preset: (), keystrokes: &mut ::#ident_crate::KeystrokeIterator, config: &Config) -> Result<#ty_for, ::#ident_crate::FromKeystrokesError> {
            //                             ::#ident_crate::from_keystrokes_by_preset_keymap(#ty_inner::get_keymap(config), keystrokes, config)
            //                         }
            //                     }
            //                 }
            //             }).reduce(join_by_space);
            //             impl_from_keystrokes = quote! {
            //                 #impl_from_keystrokes
            //                 #impl_from_keystrokes_append
            //             };
            //
            //             if let Some(ty_inner) = ty_inner {
            //                 let ident = ident.as_ref().unwrap();
            //                 let result = quote! {
            //                     impl GetKeymap<#ident_config> for #ty_inner {
            //                         fn get_keymap<'a>(config: &'a #ident_config) -> &'a #ty {
            //                             &config.#ident
            //                         }
            //                     }
            //                 };
            //                 // dbg!(result);
            //                 println!("result: {result}");
            //                 result
            //
            //             } else {
            //                 quote!{}
            //             }
            //         })
            //         .reduce(join_by_space),
            //     _ => panic!("Struct with unnamed or unit fields are not supported."),
            // };
            // let output = quote! {
            //     #output
            //     #impl_from_keystrokes
            // };
            println!("output: {output}");
            output
        }
        _ => panic!(),
    };
    let result = TokenStream::from(output);
    result
}
