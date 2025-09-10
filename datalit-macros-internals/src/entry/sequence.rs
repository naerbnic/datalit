use syn::punctuated::Punctuated;

use crate::state::{EntryState, StateOperation};

use super::Entry;

#[derive(derive_syn_parse::Parse)]
pub struct SequenceEntry {
    #[call(Punctuated::parse_terminated)]
    entries: Punctuated<Entry, syn::Token![,]>,
}

impl StateOperation for SequenceEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        for entry in &self.entries {
            entry.apply_to(state)?;
        }
        Ok(())
    }
}
