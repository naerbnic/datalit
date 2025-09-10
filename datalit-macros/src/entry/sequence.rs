use syn::punctuated::Punctuated;

use crate::{EntryState, state::StateOperation};

use super::DataLitEntry;

#[derive(derive_syn_parse::Parse)]
pub struct DataLitEntries {
    #[call(Punctuated::parse_terminated)]
    entries: Punctuated<DataLitEntry, syn::Token![,]>,
}

impl StateOperation for DataLitEntries {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        for entry in &self.entries {
            entry.apply_to(state)?;
        }
        Ok(())
    }
}
