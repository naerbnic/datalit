# `datalit`

## Overview

`datalit` is a crate that provides the `datalit!(...)` macro to generate static data described in a fluent style. Features include:

- Compiles to static arrays
- Endianness aware
- Fluent description of offset references

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

TBD.

## Acknowledgements

TBD.
