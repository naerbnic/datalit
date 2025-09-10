//! Contains types that need to be available for the macro, but are not part of its public API.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct DataRange {
    start: usize,
    end: usize,
}

impl DataRange {
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    #[must_use]
    pub fn start(&self) -> usize {
        self.start
    }

    #[must_use]
    pub fn end(&self) -> usize {
        self.end
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

pub struct LocationMap(BTreeMap<String, DataRange>);

impl LocationMap {
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, label: String, range: DataRange) {
        let had_value = self.0.insert(label, range).is_some();
        assert!(!had_value, "Duplicate label inserted into LocationMap");
    }

    #[must_use]
    pub fn get(&self, label: &str) -> Option<DataRange> {
        self.0.get(label).copied()
    }
}

impl Default for LocationMap {
    fn default() -> Self {
        Self::new()
    }
}

type RawPatchOp = Box<dyn FnOnce(&LocationMap, &mut [u8]) -> syn::Result<()>>;

pub struct PatchOp(RawPatchOp);

impl PatchOp {
    #[must_use]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&LocationMap, &mut [u8]) -> syn::Result<()> + 'static,
    {
        Self(Box::new(f))
    }

    pub fn apply(self, location_map: &LocationMap, data: &mut [u8]) -> syn::Result<()> {
        (self.0)(location_map, data)
    }
}
