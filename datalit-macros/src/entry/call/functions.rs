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
pub struct CallExprEntry {
    call_expr: FunctionExpr,
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

struct FunctionExpr {
    name: Ident,
    args: Paren,
    func: FunctionCall,
}

impl FunctionExpr {
    #[expect(dead_code, reason = "Will remove once we know we don't need it")]
    pub fn span(&self) -> proc_macro2::Span {
        self.name.span().join(self.args.span.join()).unwrap()
    }
}

impl Parse for FunctionExpr {
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

impl ProcessCall for FunctionExpr {
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
