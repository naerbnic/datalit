mod block;
mod call;
mod labeled;
mod literal;
mod mode_change;
mod sequence;

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{LitByte, LitByteStr, LitCStr, LitInt, Result, token::Brace};

use crate::state::EntryState;

pub use self::{
    block::SubEntry,
    call::CallEntry,
    labeled::LabeledEntry,
    literal::{ByteLiteral, ByteStringLiteral, CStringLiteral, IntLiteral},
    mode_change::ModeChange,
    sequence::DataLitEntries,
};

fn new_literal_bytes_stmt(state: &mut EntryState, bytes: &[u8]) -> TokenStream {
    let data_var = state.data_var();
    let byte_literals: Vec<_> = bytes.iter().map(|b| Literal::u8_suffixed(*b)).collect();
    quote! {
        #data_var.extend_from_slice(&[#(#byte_literals),*]);
    }
}

#[derive(derive_syn_parse::Parse)]
pub enum DataLitEntry {
    #[peek(LitInt, name = "integer literal")]
    Int(IntLiteral),
    #[peek(LitByteStr, name = "byte string literal")]
    ByteStr(ByteStringLiteral),
    #[peek(LitByte, name = "byte literal")]
    Byte(ByteLiteral),
    #[peek(LitCStr, name = "C-style string literal")]
    CStr(CStringLiteral),
    #[peek(Brace, name = "braced list of entries")]
    SubEntry(SubEntry),
    #[peek_with(LabeledEntry::peek, name = "labelled entry")]
    Labelled(LabeledEntry),
    #[peek_with(CallEntry::peek, name = "call entry")]
    Call(CallEntry),
    #[peek_with(ModeChange::peek, name = "mode change")]
    ModeChange(ModeChange),
}

impl DataLitEntry {
    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
        Ok(match self {
            DataLitEntry::Int(lit) => lit.into_tokens(state)?,
            DataLitEntry::ByteStr(lit) => lit.into_tokens(state)?,
            DataLitEntry::Byte(lit) => lit.into_tokens(state)?,
            DataLitEntry::CStr(lit) => lit.into_tokens(state)?,
            DataLitEntry::SubEntry(sub_entry) => sub_entry.into_tokens(state)?,
            DataLitEntry::Labelled(labelled_entry) => labelled_entry.into_tokens(state)?,
            DataLitEntry::Call(call_entry) => call_entry.into_tokens(state)?,
            DataLitEntry::ModeChange(mode_change) => mode_change.into_tokens(state)?,
        })
    }
}
