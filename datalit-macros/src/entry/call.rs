use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Ident, Lifetime, Result, parse::ParseStream, token::Paren};

use crate::{parse::base::PrimitiveSpec, state::EntryState};

pub struct CallEntry {
    _name: Ident,
    _call_args: Paren,
    call: Call,
}

impl CallEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(Paren)
    }

    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
        match self.call {
            Call::Start(start_call) => start_call.into_tokens(state),
        }
    }
}

impl syn::parse::Parse for CallEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let paren_contents;
        let call_args: Paren = syn::parenthesized!(paren_contents in input);
        let call = Call::parse(name.clone(), &paren_contents)?;
        Ok(CallEntry {
            _name: name,
            _call_args: call_args,
            call,
        })
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct StartCall {
    spec: PrimitiveSpec,
    #[prefix(syn::Token![,])]
    lifetime: Lifetime,
    _trailing: Option<syn::Token![,]>,
}

impl StartCall {
    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
        state.report_label_use(&self.lifetime);
        let crate_name = state.crate_name();
        let data_var = state.data_var();
        let patch_ops_var = state.patch_ops_var();
        let lifetime_str = syn::LitStr::new(&self.lifetime.ident.to_string(), Span::call_site());
        let int_type = self.spec.int_type();
        let data_size = int_type.to_byte_size();
        let target_type = int_type.to_type();
        let bytes_func = self
            .spec
            .endianness()
            .unwrap_or(state.endian_mode())
            .to_func_name();
        Ok(quote! {{
            let curr_offset = #data_var.len();
            #data_var.extend_from_slice(&[0u8; #data_size]);
            #patch_ops_var.push(#crate_name::support::PatchOp::new(move |loc_map, data| {
                let offset = loc_map.get_or_panic(#lifetime_str).start();
                let offset_cast: #target_type = offset.try_into().expect("Offset too large for target type");
                let source_bytes: [u8; _] = offset_cast.#bytes_func();
                data[curr_offset..][..#data_size].copy_from_slice(&source_bytes);
            }));
        }})
    }
}

enum Call {
    Start(StartCall),
}

impl Call {
    pub fn parse(name: Ident, args: ParseStream) -> Result<Self> {
        let name_str = name.to_string();
        Ok(match name_str.as_str() {
            "start" => Call::Start(args.parse()?),
            _ => {
                return Err(Error::new_spanned(
                    &name,
                    format!("Unknown call: '{}'", name),
                ));
            }
        })
    }
}
