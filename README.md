# `datalit`

## Overview

`datalit` provides the `datalit!(...)` macro to turn a readable list of things
into real bytes—at compile time. Highlights:

- _Readable data_: Hard to read raw byte arrays? Describe intent with readable
  literals.
- _Endian aware_: Unsure about byte order? Declare it once; the macro
  handles the rest.
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

TBD.

## License

This project is dual-licensed under either the MIT or Apache 2.0 license, at
your option.

- See [LICENSE-MIT](./LICENSE-MIT) for the MIT license text.
- See [LICENSE-APACHE](./LICENSE-APACHE) for the Apache 2.0 license text.

You may use this project under the terms of either license.

## Acknowledgements

TBD.

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
