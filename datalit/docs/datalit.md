The `datalit!()` macro can be used as an expression to turn a fluent
description of a block of data into a static byte array.

# Example

This is the start of a simple PNG file header and block format:

```rust
# use datalit::datalit;
let png_data = datalit!(
  // Unannotated hex literals are interpreted as raw bytes.
  0x89,
  // Binary strings are translated directly.
  b"PNG",
  0x0D0A_1A_0A,

  // PNG Chunk:

  // Data length is big endian u32.
  //
  // This is resolved from the length in bytes from the upcomming chunk.
  len('chunk1): u32_be,

  // You can set the endian mode to avoid redundancy
  @endian_mode = be,

  // The data type is a 4 byte character ID
  b"IHDR",
  'chunk1: {
    // Width
    256u32,
    // Height
    256u32,

    // Bit depth
    16u8,
    // Color type
    0u8,
    // Filter, Interlace
    0u8, 0u8
  },
  // The CRC. Not supported directly.
  0xDEADBEEF,
);

assert_eq!(png_data, &[0u8]);
```

## Entries

The contents of `datalit!()` are a sequence of individual entries which define
data that will be added to the buffer in the order provided. The different entry
types are:

- **Untyped Hex Literals**: `0x0123_4567_89AB_CDEF`

  These represent the exact bytes provided, as read from left to right. These
  have no maximum length even if the data is larger than a
  machine-representable number, but there must be a multiple of 2 hexidecimanl
  digits.

- **Untyped Binary Literals**: `0b00111100`

  These follow the same principles as untyped hex literals, but their digits
  must be a multiple of 8 instead.

- **Typed integer literals**: `12u32le`

  These are integer literals that will be written at the current location in
  the byte order specified in the annotated type, or the current endian mode
  if the endianness is not specified. The type must be provided. All native
  rust types are supported, both signed and unsigned variants. In addition,
  we support the `u24` suffix to represent a 3-byte integer as well.

- **Byte literals**: `b'X'`

  The given byte is written in the data directly.

- **Byte string literals**: `b"TIFF"`

  The byte sequence is written in the data directly.

- **C-string literals**: `c"Hello, world!\n"`

  These operate similarly to byte strings, but also write the trailing null
  into the data.

- **Entry labels**: `'data: b"some data"`

  The labeled entry is written into the data as written, but the bounds of that
  data are recorded for use in other expressions. See the expressions section
  for more information.

- **Blocks**: `{ 1u32, 2u32, }`

  A block writes its contents into the data in the order provided. This can be
  used for visual grouping. When a block is labeled, the bounds of the label
  span from before the block to after the block.

- **Simple arrays**: `[ 0u8; 100 ]`

  Any array writes its single value entry the number of times specified by the
  length (after the semicolon). The length is interpreted as it's explicit
  value, and must not be suffixed with a specific type.

- **Compound arrays**: `[{ 1u8, 2u32 }; 20]`

  Repeats its contents like simple arrays, but allows any number of entries
  within the braces. While expressions from within the array can reference
  labels, no labels can be defined inside of the braces.

- **Align**: `align(4)`

  Aligns the current data location to the given number. The number must be a
  power of two. If the current data location is already aligned, this does
  nothing.

- **Expression**: `start('data): u32`

  Gives an expression that can be evaluated to determine the value. This is
  used to interact with labels. See the documentation on expressions for more
  information.

## Expressions

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
- Syntax: Have a paren-wrapped entry be treated as an expression (with
  function calling as a special case)
