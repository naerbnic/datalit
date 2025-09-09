use proc_macro2::TokenStream;
use syn::token::Brace;

use crate::DataLitEntries;

#[derive(derive_syn_parse::Parse)]
pub struct SubEntry {
    #[brace]
    _brace_token: Brace,

    #[inside(_brace_token)]
    entries: DataLitEntries,
}

impl SubEntry {
    pub fn into_tokens(self, state: &mut crate::EntryState) -> syn::Result<TokenStream> {
        self.entries.into_tokens(state)
    }
}
