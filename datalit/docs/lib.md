This crate defines the [`datalit!`] procedural macro, which generates
static byte slices from a declarative syntax.

# Features

- Readable declarative syntax: hex & binary blobs, typed ints, bytes, strings, blocks.
- Relative offsets & forward refs: `start/end/len('label)` auto‑update when
  layout changes.
- Mixed endianness: per‑value suffixes or a persistent `@endian = le|be|ne` mode.
- 24‑bit + standard integer widths, arrays (`[x; N]` & compound), alignment
  with `align(N)`.
- Convenience literals: C‑strings (auto null), large underscored numbers, repetition.
- Zero‑cost & `no_std`: expands to a static byte slice; all validation at
  compile time.

Quick taste:

```rust
# use datalit::datalit;
let header = datalit!(0xFF, 42u16_le, b"OK", align(4));
```

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
