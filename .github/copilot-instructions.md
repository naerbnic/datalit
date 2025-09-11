# Copilot Instructions for `datalit`

Concise, project-specific guidance to make agents productive here. Keep changes minimal and aligned with existing patterns.

## Big picture
- Workspace has three crates:
  - `datalit/` (no_std library): public entry point; exports the `datalit!(...)` macro; forbids `unsafe`.
  - `datalit-macros/` (proc-macro): thin shim calling internals.
  - `datalit-macros-internals/` (library): parser + generator.
- Flow: parse entries → build `EntryState` (bytes, labels, deferred patches) → emit `'static [u8]` at compile time.
- All validation at expand time; errors via `syn::Error`.

## Dev workflows
- Build/test from root: `cargo build --workspace`, `cargo test --workspace` (most tests in `datalit/src/lib.rs`).
- Docs: update `datalit/docs/*.md` and top-level `README.md`; crate docs use `#![doc = include_str!(...)]`.
- Toolchain: `edition = "2024"` on stable.

## Key files
- API surface: `datalit/src/lib.rs` (re-exports macro; no other public items).
- Proc-macro entry: `datalit-macros/src/lib.rs` → `datalit-macros-internals::generate_expr_raw`.
- Internals:
  - Entries/parsing: `datalit-macros-internals/src/entry/*.rs`, `parse/*`.
  - State: `.../state.rs` (+ `state/support.rs`).
  - Integer/endianness: `.../to_bytes.rs` (u24/i24; le/be/ne).

## Conventions
- No `unsafe`; `datalit/` is `#![no_std]` (don’t add `std` there).
- Use `derive_syn_parse` with `peek` helpers; register variants via `build_variant!` in `entry.rs`.
- Emit errors with `syn::Error::new_spanned(...)`; combine multiples in `EntryState::check`.
- Endianness: global mode via `@endian = le|be|ne`; explicit suffixes override (`u32_le`, `i16be`, `u32ne`).

## Byte building pattern
- Direct literals (ints/hex/bin/byte/strings): `EntryState.append_bytes`.
- Computed values (e.g., `start('lbl): u32_be`): `advance_bytes(num)`, then `defer_patch_op` to write using `LocationMap`.
- Labels: record start/end; forward refs ok; duplicates error.
- Arrays: label context is frozen inside repeats (see `repeat.rs`).

## Supported entries (quick refs)
- Hex/bin literals: `0xDEADBEEF`, `0b0011_0101` (whole bytes only).
- Typed ints: `1u16`, `100u16_le`, `0x01_02_03u24_be`, `-1i24_be`.
- Bytes/strings: `b'X'`, `b"PAY"`, `c"CSTR"` (adds trailing `\0`).
- Blocks/labels: `'hdr: { 1u16, b"AB" }`; `start('hdr): u32_be`, `len('hdr): u16`.
- Arrays: `[ 0xFF; 4 ]`, `[{ 1u8, align(2), 2u8 }; 2]`.
- Directive: `align(4)` (power of two; pads 0x00).

## Extending safely
- New directive: add in `entry/call/directives.rs`; implement `StateOperation`.
- New function call: extend `entry/call/functions.rs`; return `EvalCallBox` used in a deferred patch.
- New entry kind: create module in `entry/`, implement `peek`/`Parse`/`StateOperation`, register via `build_variant!`.
- Integers: extend `IntType`/`Endianness` in `to_bytes.rs`; update docs/tests accordingly.

## Testing and guardrails
- Add tests in `datalit/src/lib.rs`; assert exact byte output. Use `#[ignore]` for known upstream issues (see README/docs example).
- Preserve public API: only the `datalit!` macro; keep `no_std` and zero runtime cost.
- When syntax/behavior changes, update docs (`datalit/docs/datalit.md`, `README.md`) and tests in the same PR.

## Maintenance
- Keep this file current: when you change syntax, add/remove entry kinds/directives/functions, tweak endianness, or adjust error messages, update this file in the same PR.
- Mirror examples from tests in `datalit/src/lib.rs` to avoid drift; if a test changes behavior/output, reflect that here.
- Align with docs: any user-facing behavior change must also update `datalit/docs/datalit.md` and `README.md`, and then be reflected here.
- Be concise (20–50 lines) and prune outdated notes promptly; do not document future ideas—only current behavior.
- If you notice discrepancies while working, proactively fix them here (don’t wait for a separate cleanup).
