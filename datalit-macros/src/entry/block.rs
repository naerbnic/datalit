use syn::token::Brace;

use crate::{DataLitEntries, EntryState, state::StateOperation};

#[derive(derive_syn_parse::Parse)]
pub struct SubEntry {
    #[brace]
    _brace_token: Brace,

    #[inside(_brace_token)]
    entries: DataLitEntries,
}

impl SubEntry {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Brace)
    }
}

impl StateOperation for SubEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        self.entries.apply_to(state)
    }
}
