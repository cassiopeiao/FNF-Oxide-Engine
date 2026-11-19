//! Proc macros for mods built against the Oxide Engine SDK.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Type, parse_macro_input};

/// Exports the mod entry point the engine looks for in the wasm component.
///
/// ```ignore
/// oesdk::register_mod!(MyMod);
/// ```
#[proc_macro]
pub fn register_mod(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn __oesdk_make_mod() -> *mut dyn ::oesdk::Mod {
            Box::into_raw(Box::new(<#ty as ::core::default::Default>::default()))
        }
    }
    .into()
}
