use syn::token::Bracket;

use crate::{
    entry::{BlockEntry, literal::LiteralEntry},
    state::StateOperation,
};

#[derive(derive_syn_parse::Parse)]
enum Contents {
    #[peek_with(BlockEntry::peek, name = "sequence of entries")]
    Braced(super::BlockEntry),
    #[peek_with(LiteralEntry::peek, name = "single literal")]
    SingleLiteral(LiteralEntry),
}

impl StateOperation for Contents {
    fn apply_to(&self, state: &mut crate::state::EntryState) -> syn::Result<()> {
        match self {
            Contents::Braced(seq) => seq.apply_to(state),
            Contents::SingleLiteral(lit) => lit.apply_to(state),
        }
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct RepeatEntry {
    #[bracket]
    _brackets: Bracket,
    #[inside(_brackets)]
    contents: Contents,
    #[prefix(syn::Token![;] in _brackets)]
    #[inside(_brackets)]
    count: syn::LitInt,
}

impl RepeatEntry {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Bracket)
    }
}

impl StateOperation for RepeatEntry {
    fn apply_to(&self, state: &mut crate::state::EntryState) -> syn::Result<()> {
        if !self.count.suffix().is_empty() {
            return Err(syn::Error::new_spanned(
                &self.count,
                "suffixes are not allowed in repeat counts",
            ));
        }
        let count: usize = self.count.base10_parse()?;
        state.freeze_label_context();
        for _ in 0..count {
            self.contents.apply_to(state)?;
        }
        state.unfreeze_label_context();
        Ok(())
    }
}
