use proc_macro::TokenStream as BaseTokenStream;

/// The `datalit` macro allows generating embedded data
#[proc_macro]
pub fn datalit(input: BaseTokenStream) -> BaseTokenStream {
    datalit_macros_internals::generate_expr_raw(input.into()).into()
}
