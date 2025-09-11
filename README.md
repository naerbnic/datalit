# `datalit`

## Overview

`datalit` is a crate that provides the `datalit!(...)` macro to generate static data described in a fluent style. Features include:

- _Readable_: Multiple ways to express data in an easy to understand way.
  Choose the one clearest for your use case!
- _Endian aware_: Working with native data? A file format with a specified
  endianness? Different endianness needed in different locations? You can
  declare individual items as being a particular endianness, or declare a
  default to use.
- _Cross referenced data_: Need to reference the location of data in a file?
  You can reference the location without having to count the bytes! References
  are adjusted automatically even if their locations change. Works with both
  forward and backward references.
- _Compile time_: Compiled macros are identical to static byte slices! They can
  be used in constant and `no_std` environments.

This can be used to make easily readable and maintainable data in tests.

## Installation

`datalit` can be added using the standard `cargo add` command:

```shell
$ cargo add --git https://github.com/naerbnic/datalit.git datalit
...
```

## Usage

In your code, you can use a datalit macro as an expression. This will resolve
to a reference to a static array that contains the data described by the
contents of the macro.

## Examples

```rust
let data = datalit!(
  // Hexideciaml literals can be of any length. Bytes are used in left-to-right
  // order.
  0xDEAD_BEEF_CAFE_BABE,

  // So can binary literals
  0b0110_1111_1000_0100,

  // Adding a primitive integer suffix creates a value of that type. By default,
  // it uses native byte order
  1u32, 0x1FFu16

  // You can annotate it with a byte order spec (one of le, be, or ne) to change
  // the byte order of the expression.
  100_u16_le,

  // You can set the endian mode to change the current default
  @endian_mode = le,
  99u16,

  // There are a few nonstandard types for convenience
  199687u24,

  // Byte string literals and byte literals translate to their natural values
  b"abcde\66",
  b'X',

  // C string literals translate to a null-terminated ASCII string.
  c"Hello, world!",

  // Labels can be used to reference the offset of an expression in the datalit
  // expression. start('label) and end('label) write the value with the given
  // data format.
  start(u16, 'buffer),
  end(u16, 'buffer),

  // Braces can be used to group multiple entries. Labels on them reference the
  // entire range of data.
  'buffer {
    12u16,
    b"quux",
  }
);
```

## Contributing

TBD.

## License

This project is dual-licensed under either the MIT or Apache 2.0 license, at your option.

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
