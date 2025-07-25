use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Provides implementation for `AsRef<dyn chrisomatic_step::PendingStep + 'static>`.
#[proc_macro_derive(AsRefPendingStep)]
pub fn as_ref_pending_step(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let out = quote! {
        impl AsRef<(dyn chrisomatic_step::PendingStep + 'static)> for #name {
            fn as_ref(&self) -> &(dyn chrisomatic_step::PendingStep + 'static) {
                self
            }
        }
    };

    out.into()
}
