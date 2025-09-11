//! This is an internal implementation detail of the
//! [`datalit`](https://docs.rs/datalit) crate. Users should not directly depend on
//! this crate.

use proc_macro::TokenStream as BaseTokenStream;

#[proc_macro]
pub fn datalit(input: BaseTokenStream) -> BaseTokenStream {
    datalit_macros_internals::generate_expr_raw(input.into()).into()
}
