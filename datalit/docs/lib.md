This crate defines the [`datalit!`][`datalit`] procedural macro, which generates
static byte slices from a declarative syntax.

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

See the [`datalit!`][`datalit`] macro definition for more details.
