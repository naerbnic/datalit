The `datalit!()` macro can be used as an expression to turn a fluent
description of a block of data into a static byte array at compile time. This
allows you to write readable, well documented descriptions of structured binary
data while incurring no runtime cost. This can frequently be useful in testing
code that does low-level data processing and parsing, among other things.

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

## Quick Reference

- Integer Types
  - Unsigned Types: `u8`, `u24`
  - Signed Types: `i8`, `i64`
  - Endianness: `u16_le`,
- Entries
  - Literals
    - Untyped hex/binary: `0xABDE`, `0b0010_1111`
    - Typed int: `123u32`
    - Byte: `b'R'`
    - Byte string: `b"buffalo"`
    - C-string: `c"foo"`
  - Blocks: `{ ... }`
  - Simple Arrays: `[ 0x00; 10 ]`
  - Compound Arrays: `[{ ... }; 5]`
  - Aligns: `align(8)`
  - Mode Change: `@endian = be`
  - Expressions: `start('data): u16`
- Expressions:
  - Start/End: `start('data)`/`end('data)`
  - Length: `len('data)`

## Entries

The contents of `datalit!()` are a sequence of individual entries which define
data that will be appended in the order provided. The different entry
types are:

### Untyped hex/binary literals

```rust
# use datalit::datalit;
# let data =
datalit!(
  0xABCD,
  0x0123_4567_89AB_CDEF_DEAD_BEEF,
  0b00111100,
)
# ;
```

These append the exact bytes provided, as read from left to right. These
have no maximum length even if the data is larger than a
machine-representable number, but only full bytes can be specified. For hex
literals, there must be an even number of
digits. For binary literals, the number of digits must be divisible by 8.
Digits may be arbitrarily separated by underscores.

### Typed integer literals

```rust
# use datalit::datalit;
# let data =
datalit!(
  12u32le,
  14u32be,
)
# ;
```

These are integer literals that will be written at the current location in
the byte order specified in the annotated type, or the current endian mode
if the endianness is not specified. The type must be provided. All native
rust types are supported, both signed and unsigned variants. In addition,
we support the `u24` suffix to represent a 3-byte integer as well.

### Byte literals

```rust
# use datalit::datalit;
# let data =
datalit!(b'X')
# ;
```

The given byte is written in the data directly.

### Byte string literals

```rust
# use datalit::datalit;
# let data =
datalit!(b"TIFF")
# ;
```

The byte sequence is written in the data directly.

### C-string literals

```rust
# use datalit::datalit;
# let data =
datalit!(c"Hello, world!\n")
# ;
```

These operate similarly to byte strings, but also append a trailing null. An
intervening null byte will not terminate the string, and
the rest of the string will be appended along with the implicit null byte.

### Entry labels

```rust
# use datalit::datalit;
# let data =
datalit!('data: b"some data")
# ;
```

The labeled entry is appended as through it were by itself, but the bounds of
the appended data are recorded for use in other expressions. See the
expressions section for more information.

### Blocks

```rust
# use datalit::datalit;
# let data =
datalit!({ 1u32, 2u32, })
# ;
```

A block appends its contents in the order provided. This can be
used for visual grouping. When a block is labeled, the bounds of the label
span from before the block to after the block.

### Simple arrays

```rust
# use datalit::datalit;
# let data =
datalit!([ 0u8; 100 ])
# ;
```

Simple arrays of the form `[ entry; N ]` will repeat the entry exactly `N`
times. N must be an unsuffixed integer literal (underscores allowed).

### Compound arrays

```rust
# use datalit::datalit;
# let data =
datalit!([{ 1u8, 2u32 }; 20])
# ;
```

Repeats its contents like simple arrays, but allows any number of entries
within the braces. While expressions from within the array can reference
labels, no labels can be defined inside of the braces.

### Align

```rust
# use datalit::datalit;
# let data =
datalit!(42u24, align(4))
# ;
```

Aligns the current data location to the given number. The number must be a
power of two. If the current data location is already aligned, this does
nothing. The bytes will be filled with zero bytes (`0x00`).

### Mode changes

```rust
# use datalit::datalit;
# let data =
datalit!(
  1u32, // Generated based on the native platform byte order
  @endian = le,
  1u32, // Generated as little endian (0x0100_0000)
  @endian = be,
  1u32, // Generated as big endian (0x0000_0001)
  @endian = ne,
  1u32, // Back to native platform byte order
)
# ;
```

During the evaluation of a datalit, to avoid excessive repetition, you can
change the defaults of the generation logic. For example, there is a default
endian mode for typed integer literals that do not provide an explicit endian
suffix. When a mode is set, that default will hold until another mode is set.

### Expression

```rust
# use datalit::datalit;
# let data =
datalit!(
  start('data): u32,
  'data: b"buffalo",
)
# ;
```

Gives an expression that can be evaluated to determine the value. This is
used to interact with labels. See the documentation on expressions for more
information.

## Entry Sequences

In both the body of the top-level macro, as well as blocks, entries are
separated by commas. Commas are required between any two entries. Trailing
commas are permitted.

## Expressions

## Guarantees

- **Fully const**: The generated data is entirely produced at compile time, and
  the resulting values are usable in const contexts.
- **Deterministic**: The generation process ensures that the generated data is
  identical from run to run.
- **`nostd` compatible**: The generated array is static, and does not depend on
  an allocator.

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
