use syn::{Lifetime, parse::ParseStream};

use crate::state::{EntryState, StateOperation};

use super::Entry;

#[derive(derive_syn_parse::Parse)]
pub struct LabeledEntry {
    label: Lifetime,
    #[prefix(syn::Token![:])]
    sub_entry: Box<Entry>,
}

impl LabeledEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Lifetime) && input.peek2(syn::Token![:])
    }
}

impl StateOperation for LabeledEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        let start = state.curr_offset();
        self.sub_entry.apply_to(state)?;
        let end = state.curr_offset();
        state.report_label_def(&self.label, start, end)
    }
}
