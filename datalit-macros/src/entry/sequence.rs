use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use crate::EntryState;

use super::DataLitEntry;

#[derive(derive_syn_parse::Parse)]
pub struct DataLitEntries {
    #[call(Punctuated::parse_terminated)]
    entries: Punctuated<DataLitEntry, syn::Token![,]>,
}

impl DataLitEntries {
    pub fn into_tokens(self, state: &mut EntryState) -> syn::Result<TokenStream> {
        let mut data_statements = Vec::new();
        for entry in self.entries {
            data_statements.push(entry.into_tokens(state)?);
        }
        Ok(quote! {
            {
                #(#data_statements)*
            }
        })
    }
}
