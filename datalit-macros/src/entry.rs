use proc_macro2::{Literal, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt as _, format_ident, quote};
use syn::{
    Error, Ident, Lifetime, LitByte, LitByteStr, LitCStr, LitInt, Result,
    parse::ParseStream,
    punctuated::Punctuated,
    token::{Brace, Paren},
};

use crate::{
    entry_state::EntryState,
    to_bytes::{Endianness, IntType, base10_digits_to_bytes},
};

pub struct PrimitiveSpec {
    ident: Ident,
    int_type: IntType,
    endianness: Option<Endianness>,
}

impl syn::parse::Parse for PrimitiveSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let ident_string = ident.to_string();
        let mut suffix = ident_string.as_str();

        let endianness = if suffix.ends_with("le") {
            suffix = suffix.trim_end_matches("le");
            suffix = suffix.trim_end_matches('_');
            Some(Endianness::Little)
        } else if suffix.ends_with("be") {
            suffix = suffix.trim_end_matches("be");
            suffix = suffix.trim_end_matches('_');
            Some(Endianness::Big)
        } else if suffix.ends_with("ne") {
            suffix = suffix.trim_end_matches("ne");
            suffix = suffix.trim_end_matches('_');
            Some(Endianness::Native)
        } else {
            None
        };

        let int_type = IntType::from_suffix(suffix).ok_or_else(|| {
            Error::new_spanned(
                &ident,
                format!("Invalid or missing integer type suffix: '{}'", ident),
            )
        })?;

        Ok(PrimitiveSpec {
            ident,
            int_type,
            endianness,
        })
    }
}

impl quote::ToTokens for PrimitiveSpec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(self.ident.clone());
    }
}

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

    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
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
        Ok(quote! {})
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct DataLitEntries {
    #[call(Punctuated::parse_terminated)]
    entries: Punctuated<DataLitEntry, syn::Token![,]>,
}

impl DataLitEntries {
    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
        let mut data_statements = Vec::new();
        for entry in self.entries {
            data_statements.push(entry.into_tokens(state)?);
        }
        Ok(quote! {
            {
                #(#data_statements)*
            }
        })
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct LabelledEntry {
    label: Lifetime,
    #[prefix(syn::Token![:])]
    sub_entry: Box<DataLitEntry>,
}

impl LabelledEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Lifetime) && input.peek2(syn::Token![:])
    }

    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
        state.report_label_def(&self.label)?;
        let statements = self.sub_entry.into_tokens(state)?;
        let data_var = state.data_var();
        let crate_name = state.crate_name();
        let loc_map_var = state.loc_map_var();
        let label_start = format_ident!("__{}_start", self.label.ident);
        let label_end = format_ident!("__{}_end", self.label.ident);
        let data_range = format_ident!("__{}_range", self.label.ident);
        let label_str = syn::LitStr::new(&self.label.ident.to_string(), self.label.span());
        Ok(quote! {
            {
                let #label_start: usize = #data_var.len();
                #statements
                let #label_end: usize = #data_var.len();
                let #data_range = #crate_name::support::DataRange::new(#label_start, #label_end);
                #loc_map_var.insert(#label_str.to_string(), #data_range);
            }
        })
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct SubEntry {
    #[brace]
    _brace_token: syn::token::Brace,

    #[inside(_brace_token)]
    entries: DataLitEntries,
}

fn parse_byte_literal<T>(
    err_context: &T,
    digit_type_name: &str,
    digits: &str,
    digits_per_byte: usize,
) -> Result<Vec<u8>>
where
    T: ToTokens,
{
    // This should be a valid hex string, which should be in ascii.
    // We can use the byte length to determine how many chars were used.
    // We need an even number of hex digits to form bytes.
    assert_eq!(8 % digits_per_byte, 0);
    if digits.len() % digits_per_byte != 0 {
        return Err(Error::new_spanned(
            err_context,
            format!(
                "{} literal must have an even number of digits to form bytes. Has {} digits.",
                digit_type_name,
                digits.len()
            ),
        ));
    }
    (0..digits.len())
        .step_by(digits_per_byte)
        .map(|i| {
            u8::from_str_radix(
                &digits[i..i + digits_per_byte],
                2u32.pow((8 / digits_per_byte) as u32),
            )
        })
        .collect::<std::result::Result<Vec<u8>, _>>()
        .map_err(|e| {
            Error::new_spanned(
                err_context,
                format!("Invalid {} literal: {e}", digit_type_name.to_lowercase()),
            )
        })
}

fn parse_int_literal(default_endianness: Endianness, lit: LitInt) -> Result<Vec<u8>> {
    let mut suffix = lit.suffix();

    if suffix.is_empty() {
        // Check to see if the representation is a hexidecimal literal
        let literal_digits = lit.to_string().to_ascii_lowercase();
        if literal_digits.starts_with("0x") {
            // This should be a valid hex string, which should be in ascii.
            // We can use the byte length to determine how many chars were used.
            // We need an even number of hex digits to form bytes.
            let hex_digits = literal_digits.trim_start_matches("0x").replace('_', "");
            return parse_byte_literal(&lit, "Hex", &hex_digits, 2);
        } else if literal_digits.starts_with("0b") {
            let bin_digits = literal_digits.trim_start_matches("0b").replace('_', "");
            return parse_byte_literal(&lit, "Binary", &bin_digits, 8);
        } else {
            return Err(Error::new_spanned(
                &lit,
                "Integer literal must have a type suffix (e.g. 'u8', 'i32', etc.) or be a hex (0x...) or binary (0b...) literal",
            ));
        }
    }

    let endianness = if suffix.ends_with("le") {
        suffix = suffix.trim_end_matches("le");
        suffix = suffix.trim_end_matches('_');
        Endianness::Little
    } else if suffix.ends_with("be") {
        suffix = suffix.trim_end_matches("be");
        suffix = suffix.trim_end_matches('_');
        Endianness::Big
    } else if suffix.ends_with("ne") {
        suffix = suffix.trim_end_matches("ne");
        suffix = suffix.trim_end_matches('_');
        Endianness::Native
    } else {
        default_endianness
    };

    let int_type = IntType::from_suffix(suffix).ok_or_else(|| {
        Error::new_spanned(
            &lit,
            format!("Invalid or missing integer type suffix: '{}'", lit.suffix()),
        )
    })?;

    base10_digits_to_bytes(lit.base10_digits(), int_type, endianness)
}

fn new_literal_bytes_stmt(state: &mut EntryState, bytes: &[u8]) -> TokenStream {
    let data_var = state.data_var();
    let byte_literals: Vec<_> = bytes.iter().map(|b| Literal::u8_suffixed(*b)).collect();
    quote! {
        #data_var.extend_from_slice(&[#(#byte_literals),*]);
    }
}

pub struct CallEntry {
    pub _name: Ident,
    pub _call_args: Paren,
    pub call: Call,
}

impl CallEntry {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(Paren)
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
        let data_size = self.spec.int_type.to_byte_size();
        let target_type = self.spec.int_type.to_type();
        let bytes_func = self
            .spec
            .endianness
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

pub enum Call {
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

#[derive(derive_syn_parse::Parse)]
pub enum DataLitEntry {
    #[peek(LitInt, name = "integer literal")]
    Int(LitInt),
    #[peek(LitByteStr, name = "byte string literal")]
    ByteStr(LitByteStr),
    #[peek(LitByte, name = "byte literal")]
    Byte(LitByte),
    #[peek(LitCStr, name = "C-style string literal")]
    CStr(LitCStr),
    #[peek(Brace, name = "braced list of entries")]
    SubEntry(SubEntry),
    #[peek_with(LabelledEntry::peek, name = "labelled entry")]
    Labelled(LabelledEntry),
    #[peek_with(CallEntry::peek, name = "call entry")]
    Call(CallEntry),
    #[peek_with(ModeChange::peek, name = "mode change")]
    ModeChange(ModeChange),
}

impl DataLitEntry {
    pub fn into_tokens(self, state: &mut EntryState) -> Result<TokenStream> {
        Ok(match self {
            DataLitEntry::Int(lit) => {
                let bytes: Vec<_> = parse_int_literal(state.endian_mode(), lit)?;
                new_literal_bytes_stmt(state, &bytes)
            }
            DataLitEntry::ByteStr(lit) => {
                let bytes = lit.value();
                new_literal_bytes_stmt(state, &bytes)
            }
            DataLitEntry::Byte(lit) => {
                let data_var = state.data_var();
                let byte = lit.value();
                quote! { #data_var.push(#byte); }
            }
            DataLitEntry::CStr(lit) => {
                let c_string = lit.value();
                let bytes = c_string.as_bytes_with_nul();
                new_literal_bytes_stmt(state, bytes)
            }
            DataLitEntry::SubEntry(sub_entry) => sub_entry.entries.into_tokens(state)?,
            DataLitEntry::Labelled(labelled_entry) => labelled_entry.into_tokens(state)?,
            DataLitEntry::Call(call_entry) => match call_entry.call {
                Call::Start(start_call) => start_call.into_tokens(state)?,
            },
            DataLitEntry::ModeChange(mode_change) => mode_change.into_tokens(state)?,
        })
    }
}
