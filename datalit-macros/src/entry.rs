mod block;
mod call;
mod labeled;
mod literal;
mod mode_change;
mod sequence;

use crate::state::{EntryState, StateOperation};

pub use self::{
    block::SubEntry,
    call::CallEntry,
    labeled::LabeledEntry,
    literal::{ByteLiteral, ByteStringLiteral, CStringLiteral, IntLiteral},
    mode_change::ModeChange,
    sequence::DataLitEntries,
};

macro_rules! build_variant {
    (enum $name:ident {$(
        ($type:ident, $desc:literal)
    ),* $(,)?}) => {
        // Define the enum with variants and associated parsing logic
        #[derive(derive_syn_parse::Parse)]
        pub enum $name {
            $(
                #[peek_with($type::peek, name = $desc)]
                $type($type),
            )*
        }

        impl StateOperation for $name {
            fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
                match self {
                    $(
                        $name::$type(inner) => inner.apply_to(state)?,
                    )*
                }
                Ok(())
            }
        }
    }
}

build_variant! {
    enum DataLitEntry {
        (IntLiteral, "integer literal"),
        (ByteStringLiteral, "byte string literal"),
        (ByteLiteral, "byte literal"),
        (CStringLiteral, "C-style string literal"),
        (SubEntry, "braced list of entries"),
        (LabeledEntry, "labelled entry"),
        (CallEntry, "call entry"),
        (ModeChange, "mode change"),
    }
}
