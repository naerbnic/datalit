#![doc(hidden)]

use proc_macro::TokenStream as BaseTokenStream;

#[proc_macro]
pub fn datalit(input: BaseTokenStream) -> BaseTokenStream {
    datalit_macros_internals::generate_expr_raw(input.into()).into()
}
