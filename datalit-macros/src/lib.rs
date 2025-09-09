mod entry;
mod entry_state;
mod to_bytes;

use proc_macro::TokenStream as BaseTokenStream;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{entry::DataLitEntries, entry_state::EntryState};

const BASE_CRATE: &str = "datalit";

#[proc_macro]
pub fn datalit(input: BaseTokenStream) -> BaseTokenStream {
    datalit_impl(input.into())
        .unwrap_or_else(|e| {
            let errors = e.into_iter().map(syn::Error::into_compile_error);
            quote! { {#(#errors);*}}
        })
        .into()
}

fn datalit_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let entries: DataLitEntries = syn::parse2(input)?;

    let mut state = EntryState::new();
    let contents = entries.into_tokens(&mut state)?;
    state.check()?;
    Ok(state.generate_expr(contents))
}
