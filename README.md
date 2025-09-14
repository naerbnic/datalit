# `datalit`

[![Crates.io](https://img.shields.io/crates/v/datalit.svg)](https://crates.io/crates/datalit)
[![docs.rs](https://docs.rs/datalit/badge.svg)](https://docs.rs/datalit)
[![CI](https://github.com/naerbnic/datalit/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/naerbnic/datalit/actions/workflows/ci.yaml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/naerbnic/datalit#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.89-blue)](./CONTRIBUTING.md#toolchains--msrv)

## Overview

`datalit` provides the `datalit!(...)` macro to turn a readable list of things
into real bytes—at compile time. Highlights:

- _Readable data_: Hard to read raw byte arrays? Describe intent with readable
  literals.
- _Endian aware_: Can't read bytes backwards? Declare the endianness; the
  macro handles the rest.
- _Offsets_: Tired of recalculating offsets? Labels and
  offset expressions update themselves.
- _Concise_: Spending time on padding & length management? Built-ins remove
  the manual bookkeeping.
- _Zero cost at runtime_: Worried about hidden cost or mistakes? Your data is
  validated at compile time, and expands to one static slice.

## Installation

`datalit` can be added using the standard `cargo add` command:

```shell
$ cargo add datalit
...
```

## Usage

In your code, you can use a datalit macro as an expression. This will resolve
to a reference to a static array that contains the data described by the
contents of the macro.

## Examples

```rust
use datalit::datalit;

let data = datalit!(
  // Hex / binary literals of any length (whole bytes) appended L->R.
  0xDEAD_BEEF_CAFE_BABE,
  0b0110_1111_1000_0100,

  // Primitive integers: native endian by default.
  1u32, 0x1FFu16,

  // Explicit endianness via suffix or mode.
  100u16_le,
  @endian = be,
  42u32,           // big-endian now

  // Non-standard width.
  0x01_02_03u24_be,

  // Strings / bytes.
  b"quux", b'X', c"Hello, world!",

  // Alignment to next multiple of 8 (pads with 0x00)
  align(8),

  // A labeled block and offset expressions.
  start('payload): u16_le,
  'payload: {
    12u16,
    b"PAY",
  },
  end('payload): u16_le,
  len('payload): u16_le,

  // Simple & compound arrays.
  [ 0xFF; 4 ],
  [{ 0xAA, 0xBB }; 2],
);
assert!(data.len() > 0);
```

## Contributing

Contributions are welcome! If you’ve got a bug report, idea, or small fix:

- Use the issue templates to file bugs or propose features.
- For small docs or code tweaks, open a PR directly.
- For larger changes (new syntax, behavior, or breaking changes), please open an issue or discussion first so we can align on design.

Before you open a PR, please:

- Run formatting and lints (we deny warnings) and the test/docs suites.
- Keep the public crate `no_std` and `forbid(unsafe_code)` guarantees intact.
- Add or update tests and docs for user-visible changes.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details on the workflow and guidelines.

## License

This project is dual-licensed under either the MIT or Apache 2.0 license, at
your option.

- See [LICENSE-MIT](./LICENSE-MIT) for the MIT license text.
- See [LICENSE-APACHE](./LICENSE-APACHE) for the Apache 2.0 license text.

You may use this project under the terms of either license.

## Security

Please do not report vulnerabilities in public issues. For private disclosure instructions, see [SECURITY.md](./SECURITY.md).

## Acknowledgements

Thanks to the Rust macros and tooling ecosystem—particularly `syn`, `quote`, and `proc-macro2`—for making ergonomic proc-macros possible.

And thanks in advance to contributors for bug reports, ideas, and reviews.

## Future work

- Allow for scoped labels, so they can be used in compound arrays.
- Add basic math operators, so things like relative offsets or the like can be
  computed.
- Allow for alignment to define the fill byte (or make it part of the mode)
- Implement scoped modes, so mode changes within a block can be made without
  affecting the outside state.
- Allow for some relatively common specialized operations, such as CRCs
- Allow labeled range offsets to be exported along with the data so runtime
  code can use it as needed.
- Support for strings, including with multiple encodings
- Syntax: Have a paren-wrapped entry be treated as an expression (with
  function calling as a special case)
