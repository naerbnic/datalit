use proc_macro2::Span;
use syn::{
    Error, Ident, Lifetime,
    parse::{Parse, ParseStream},
    token::Paren,
};

use crate::{
    parse::base::PrimitiveSpec,
    state::{EntryState, StateOperation, support::LocationMap},
};

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

#[derive(derive_syn_parse::Parse)]
pub struct CallExprEntry {
    call_expr: CallExpr,
    #[prefix(syn::Token![:])]
    primitive: PrimitiveSpec,
}

impl CallExprEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(Paren) && input.peek3(syn::Token![:])
    }
}

impl StateOperation for CallExprEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        let eval = self.call_expr.process(state)?;
        let curr_offset = state.curr_offset();
        let primitive = self.primitive.clone();
        let endian_mode = state.endian_mode();
        state.advance_bytes(primitive.int_type().num_bytes());
        state.defer_patch_op(move |location_map, data| {
            let value = eval.eval(location_map)?;
            assert!(data.len() >= curr_offset);
            primitive.write_int(endian_mode, &value, &mut data[curr_offset..])?;
            Ok(())
        });
        Ok(())
    }
}

trait EvalCall {
    fn eval(&self, location_map: &LocationMap) -> syn::Result<num::BigInt>;
}

impl<F> EvalCall for F
where
    F: Fn(&LocationMap) -> syn::Result<num::BigInt> + 'static,
{
    fn eval(&self, location_map: &LocationMap) -> syn::Result<num::BigInt> {
        (self)(location_map)
    }
}

struct EvalCallBox(Box<dyn EvalCall>);

impl EvalCallBox {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&LocationMap) -> syn::Result<num::BigInt> + 'static,
    {
        Self(Box::new(f))
    }

    fn eval(&self, location_map: &LocationMap) -> syn::Result<num::BigInt> {
        self.0.eval(location_map)
    }
}

trait ProcessCall {
    fn process(&self, state: &mut EntryState) -> syn::Result<EvalCallBox>;
}

#[derive(derive_syn_parse::Parse)]
pub struct StartCall {
    lifetime: Lifetime,
    _trailing: Option<syn::Token![,]>,
}

impl ProcessCall for StartCall {
    fn process(&self, state: &mut EntryState) -> syn::Result<EvalCallBox> {
        state.report_label_use(&self.lifetime);
        let lifetime_span = self.lifetime.span();
        let name = self.lifetime.ident.to_string();
        Ok(EvalCallBox::new(move |location_map: &LocationMap| {
            let range = location_map.get(&name).ok_or_else(|| {
                Error::new(lifetime_span, format!("Label '{}' not defined", name))
            })?;
            Ok(range.start().into())
        }))
    }
}

struct CallExpr {
    name: Ident,
    args: Paren,
    func: FunctionCall,
}

impl CallExpr {
    #[expect(dead_code, reason = "Will remove once we know we don't need it")]
    pub fn span(&self) -> proc_macro2::Span {
        self.name.span().join(self.args.span.join()).unwrap()
    }
}

impl Parse for CallExpr {
    fn parse(args: ParseStream) -> syn::Result<Self> {
        let name: Ident = args.parse()?;
        let arg_content;
        let args: Paren = syn::parenthesized!(arg_content in args);
        let func = match name.to_string().as_str() {
            "start" => FunctionCall::Start(StartCall::parse(&arg_content)?),
            _ => {
                return Err(Error::new_spanned(
                    &name,
                    format!("Unknown call name: '{}'", name),
                ));
            }
        };
        Ok(Self { name, args, func })
    }
}

impl ProcessCall for CallExpr {
    fn process(&self, state: &mut EntryState) -> syn::Result<EvalCallBox> {
        self.func.process(state)
    }
}

enum FunctionCall {
    Start(StartCall),
}

impl ProcessCall for FunctionCall {
    fn process(&self, state: &mut EntryState) -> syn::Result<EvalCallBox> {
        match self {
            FunctionCall::Start(start_call) => start_call.process(state),
        }
    }
}

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
