use proc_macro::TokenStream;

mod from_keystrokes;
mod get_keymap;
mod presetable;
mod utils;

#[proc_macro_derive(Presetable, attributes(presetable))]
pub fn derive_preset(input: TokenStream) -> TokenStream {
    presetable::derive_presetable(input)
}

#[proc_macro_derive(GetKeymap, attributes(preset_for))]
pub fn derive_get_keymap(input: TokenStream) -> TokenStream {
    get_keymap::derive_get_keymap(input)
}

#[proc_macro_derive(FromKeystrokes, attributes(from_keystrokes))]
pub fn derive_from_keystrokes(input: TokenStream) -> TokenStream {
    from_keystrokes::derive_from_keystrokes(input)
}
