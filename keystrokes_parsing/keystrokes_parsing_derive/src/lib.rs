use proc_macro::TokenStream;

mod presetable;
mod utils;

#[proc_macro_derive(Presetable, attributes(presetable))]
pub fn derive_preset(input: TokenStream) -> TokenStream {
    presetable::derive_presetable(input)
}
