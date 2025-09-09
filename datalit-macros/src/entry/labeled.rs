use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Lifetime, parse::ParseStream};

use crate::EntryState;

use super::DataLitEntry;

#[derive(derive_syn_parse::Parse)]
pub struct LabeledEntry {
    label: Lifetime,
    #[prefix(syn::Token![:])]
    sub_entry: Box<DataLitEntry>,
}

impl LabeledEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Lifetime) && input.peek2(syn::Token![:])
    }

    pub fn into_tokens(self, state: &mut EntryState) -> syn::Result<TokenStream> {
        state.report_label_def(&self.label)?;
        let statements = self.sub_entry.into_tokens(state)?;
        let data_var = state.data_var();
        let crate_name = state.crate_name();
        let loc_map_var = state.loc_map_var();
        let label_start = format_ident!("__{}_start", self.label.ident);
        let label_end = format_ident!("__{}_end", self.label.ident);
        let data_range = format_ident!("__{}_range", self.label.ident);
        let label_str = syn::LitStr::new(&self.label.ident.to_string(), self.label.span());
        Ok(quote! {
            {
                let #label_start: usize = #data_var.len();
                #statements
                let #label_end: usize = #data_var.len();
                let #data_range = #crate_name::support::DataRange::new(#label_start, #label_end);
                #loc_map_var.insert(#label_str.to_string(), #data_range);
            }
        })
    }
}
