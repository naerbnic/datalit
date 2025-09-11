This crate defines the [`datalit!`] procedural macro, which generates
static byte slices from a declarative syntax.

# Features

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

  // Data length is big-endian u32.
  //
  // This is resolved from the length in bytes from the upcoming chunk.
  len('chunk1): u32_be,

  // You can set the endian mode to avoid redundancy
  @endian = be,

  // The PNG chunk type is a 4-byte ASCII code.
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
```

See the [`datalit!`] macro definition for more details.
