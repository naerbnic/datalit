pub mod support;

use std::collections::{BTreeMap, btree_map::Entry};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Lifetime;

use crate::{
    state::support::{LocationMap, PatchOp},
    to_bytes::Endianness,
};

struct LabelInfo {
    source_token: Lifetime,
}

struct LabelRef {
    source_token: Lifetime,
}

pub struct EntryState {
    data: Vec<u8>,
    patch_ops: Vec<PatchOp>,
    location_map: LocationMap,
    defined_labels: BTreeMap<String, LabelInfo>,
    used_labels: BTreeMap<String, LabelRef>,
    endian_mode: Endianness,
}

impl EntryState {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            patch_ops: Vec::new(),
            location_map: LocationMap::new(),
            defined_labels: BTreeMap::new(),
            used_labels: BTreeMap::new(),
            endian_mode: Endianness::Native,
        }
    }

    pub fn report_label_def(
        &mut self,
        label: &Lifetime,
        start: usize,
        end: usize,
    ) -> syn::Result<()> {
        let label_str = label.ident.to_string();
        match self.defined_labels.entry(label_str) {
            Entry::Vacant(vacant) => {
                vacant.insert(LabelInfo {
                    source_token: label.clone(),
                });
                self.location_map
                    .insert(label.ident.to_string(), support::DataRange::new(start, end));
            }
            Entry::Occupied(occ) => {
                let mut err1 = syn::Error::new_spanned(label, "Duplicate label");
                err1.combine(syn::Error::new_spanned(
                    &occ.get().source_token,
                    "Originally defined here",
                ));

                return Err(err1);
            }
        };

        Ok(())
    }

    pub fn report_label_use(&mut self, label: &Lifetime) {
        let label_str = label.ident.to_string();
        self.used_labels.entry(label_str).or_insert(LabelRef {
            source_token: label.clone(),
        });
    }

    pub fn endian_mode(&self) -> Endianness {
        self.endian_mode
    }

    pub fn set_endian_mode(&mut self, mode: Endianness) {
        self.endian_mode = mode;
    }

    pub fn check(&self) -> syn::Result<()> {
        let mut errors = Vec::new();

        for (label_str, label_info) in &self.used_labels {
            if !self.defined_labels.contains_key(label_str) {
                errors.push(syn::Error::new_spanned(
                    &label_info.source_token,
                    format!("Label '{label_str}' used but not defined"),
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let combined_err = errors
                .into_iter()
                .reduce(|mut acc, err| {
                    acc.combine(err);
                    acc
                })
                .unwrap();
            Err(combined_err)
        }
    }

    pub fn generate_expr(&mut self) -> syn::Result<TokenStream> {
        // Apply all deferred patch operations
        for patch_op in self.patch_ops.drain(..) {
            patch_op.apply(&self.location_map, &mut self.data)?;
        }
        let byte_array = self
            .data
            .iter()
            .map(|b| syn::LitByte::new(*b, Span::call_site()));
        Ok(quote! {{
            let __slice: &'static [u8] = &[
                #(#byte_array),*
            ];
            __slice
        }})
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn advance_bytes(&mut self, n: usize) {
        let start = self.data.len();
        self.data.resize(start + n, 0);
    }

    pub fn curr_offset(&self) -> usize {
        self.data.len()
    }

    pub fn defer_patch_op<F>(&mut self, f: F)
    where
        F: FnOnce(&LocationMap, &mut [u8]) -> syn::Result<()> + 'static,
    {
        self.patch_ops.push(PatchOp::new(f));
    }
}

pub trait StateOperation {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()>;
}
