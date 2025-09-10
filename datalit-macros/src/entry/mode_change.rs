use syn::{Error, Ident, parse::ParseStream};

use crate::{EntryState, state::StateOperation, to_bytes::Endianness};

#[derive(derive_syn_parse::Parse)]
pub struct ModeChange {
    #[prefix(syn::Token![@])]
    mode: Ident,
    #[prefix(syn::Token![=])]
    new_mode: Ident,
}

impl ModeChange {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(syn::Token![@]) && input.peek2(Ident) && input.peek3(syn::Token![=])
    }
}

impl StateOperation for ModeChange {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        let mode_str = self.mode.to_string();
        if mode_str != "endian_mode" {
            return Err(Error::new_spanned(
                &self.mode,
                format!("Unknown mode: '{}'", mode_str),
            ));
        }

        let new_mode_str = self.new_mode.to_string();
        let new_mode = match new_mode_str.as_str() {
            "le" => Endianness::Little,
            "be" => Endianness::Big,
            "ne" => Endianness::Native,
            _ => {
                return Err(Error::new_spanned(
                    &self.new_mode,
                    format!("Invalid endian mode: '{}'", new_mode_str),
                ));
            }
        };
        state.set_endian_mode(new_mode);
        Ok(())
    }
}
