mod directives;
mod functions;

use syn::parse::ParseStream;

use crate::state::{EntryState, StateOperation};

use self::{directives::DirectiveEntry, functions::CallExprEntry};

#[derive(derive_syn_parse::Parse)]
pub enum CallEntry {
    #[peek_with(CallExprEntry::peek, name = "call expression")]
    CallExpr(CallExprEntry),
    #[peek_with(DirectiveEntry::peek, name = "directive")]
    Directive(DirectiveEntry),
}

impl CallEntry {
    pub fn peek(input: ParseStream) -> bool {
        CallExprEntry::peek(input) || DirectiveEntry::peek(input)
    }
}

impl StateOperation for CallEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        match self {
            CallEntry::CallExpr(call_expr) => call_expr.apply_to(state),
            CallEntry::Directive(directive) => directive.apply_to(state),
        }
    }
}
