//! This crate is used to test the `datalit` workspace documentation, when they are outside of
//! any other published crate.

// Run doctests on the README file.
//
// Note that this test will break if this crate is ever packaged, so this crate must
// always be marked `publish = false` in its Cargo.toml.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
mod readme {}
