use syn::token::Brace;

use crate::{
    entry::SequenceEntry,
    state::{EntryState, StateOperation},
};

#[derive(derive_syn_parse::Parse)]
pub struct BlockEntry {
    #[brace]
    _brace_token: Brace,

    #[inside(_brace_token)]
    entries: SequenceEntry,
}

impl BlockEntry {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Brace)
    }
}

impl StateOperation for BlockEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        self.entries.apply_to(state)
    }
}
