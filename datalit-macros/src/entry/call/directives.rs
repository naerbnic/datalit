use proc_macro2::Span;
use syn::{
    Error, Ident,
    parse::{Parse, ParseStream},
    token::Paren,
};

use crate::state::{EntryState, StateOperation};

pub struct DirectiveEntry {
    #[expect(dead_code, reason = "Will shortly be implementing directives")]
    name: Ident,
    #[expect(dead_code, reason = "Will shortly be implementing directives")]
    args: Paren,
    directive: Directive,
}

impl DirectiveEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(Paren) && !input.peek3(syn::Token![:])
    }
}

impl Parse for DirectiveEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let arg_content;
        let args: Paren = syn::parenthesized!(arg_content in input);
        let directive = Directive::parse(name.span(), &name.to_string(), &arg_content)?;
        Ok(Self {
            name,
            args,
            directive,
        })
    }
}

impl StateOperation for DirectiveEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        self.directive.apply_to(state)
    }
}

pub enum Directive {
    Align(AlignDirective),
}

impl Directive {
    fn parse(call_span: Span, name: &str, args: ParseStream) -> syn::Result<Self> {
        Ok(match name {
            "align" => Directive::Align(AlignDirective::parse(args)?),
            _ => {
                return Err(Error::new(
                    call_span,
                    format!("Unknown directive name: '{}'", name),
                ));
            }
        })
    }
}

impl StateOperation for Directive {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        match self {
            Directive::Align(align_directive) => align_directive.apply_to(state),
        }
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct AlignDirective {
    alignment: syn::LitInt,
    _trailing: Option<syn::Token![,]>,
}

impl StateOperation for AlignDirective {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        let alignment = self.alignment.base10_parse::<usize>()?;
        if !alignment.is_power_of_two() {
            return Err(Error::new_spanned(
                &self.alignment,
                "Alignment must be a power of two",
            ));
        }
        let curr_offset = state.curr_offset();
        state.advance_bytes(curr_offset.next_multiple_of(alignment) - curr_offset);
        Ok(())
    }
}
