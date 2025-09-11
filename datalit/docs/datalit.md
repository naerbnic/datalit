A macro to create a byte slice with readably described contents.

The `datalit!()` macro can be used as an expression to turn a fluent
description of a block of data into a byte slice at compile time. This
allows you to write readable, well-documented descriptions of structured binary
data while incurring no runtime cost. This is particularly useful in tests and
examples for code that performs low-level parsing or binary protocol handling.

# Usage

`datalit!()` can be used in any expression context:

```rust
# use datalit::datalit;
fn parse_buffer(data: &[u8]) {}

#[test]
fn test_data_parsing() {
  parse_buffer(datalit!(0xDEADBEEF));
}
```

They can also be used in a constant context, such as for defining a
constant variable:

```rust
# use datalit::datalit;
const HEADER: &[u8] = datalit!(0xCAFEBABE);
```

# Quick Reference

- Typed integers: `u8 u16 u24 u32 u64 u128 i8 i16 i32 i64 i128`
  (add `_le` / `_be` for explicit endianness; otherwise current endian mode /
  native; both `u32le` and `u32_le` accepted)
- Untyped hex / binary: `0xABDE`, `0b0010_1111` (must form whole bytes;
  underscores ignored)
- Byte / byte string / C-string: `b'R'`, `b"buffalo"`, `c"foo"`
  (C-string appends trailing `\0`)
- Blocks: `{ ... }` (may be labeled; label spans entire block)
- Arrays: simple `[ entry ; N ]`, compound `[{ e1, e2 }; N]`
  (no labels inside compound body)
- Align: `align(8)` (power of two; fills with `0x00`)
- Mode change: `@endian = le | be | ne` (default native `ne`; this sets the
   current endian mode)
- Expressions: `start('lbl) end('lbl) len('lbl)`
  (typed target example: `len('lbl): u32_be`)
- Labels: `'name: entry` (forward refs allowed; duplicate = error)
- Trailing commas: allowed after any entry list.

# Entries

The contents of `datalit!()` are a sequence of individual entries appended in
order. The different entry types are:

## Untyped hex / binary literals

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

These append the exact bytes provided (no runtime parsing). Length is unbounded
but only whole bytes may be formed. Hex literals must have an even number of
hex digits; binary literals a multiple of 8 binary digits. Underscores are
ignored and may appear anywhere between digits.

## Typed integer literals

```rust
# use datalit::datalit;
# let data =
datalit!(
  12u32_le,
  14u32_be,
)
# ;
```

These are integer literals appended in the specified endianness or the current
endian mode. If the suffix ends with `le` / `be` (optionally preceded by `_`)
that endianness is used; otherwise the current endian mode (`@endian`) applies
(default native). All
primitive integer widths are supported plus the non-standard `u24` (three
bytes). Example:

```rust
# use datalit::datalit;
# let _ = datalit!(
  1u16_le, 1u16_be,
  0x01_02_03u24_be, 0x01_02_03u24_le,
);
```

## Byte literals

```rust
# use datalit::datalit;
# let data =
datalit!(b'X')
# ;
```

The given byte is appended.

## Byte string literals

```rust
# use datalit::datalit;
# let data =
datalit!(b"TIFF")
# ;
```

The byte sequence is appended.

## C-string literals

```rust
# use datalit::datalit;
# let data =
datalit!(c"Hello, world!\n")
# ;
```

These operate similarly to byte strings, but also append a trailing null. An
intervening null byte is preserved; the remainder of the string is appended and
exactly one trailing null is added.

## Entry labels

```rust
# use datalit::datalit;
# let data =
datalit!('data: b"some data")
# ;
```

The labeled entry is appended as though it were by itself, but the start and
end offsets are recorded for expressions (`start`, `end`, `len`). Forward
references are allowed; redefining a label is an error.

## Blocks

```rust
# use datalit::datalit;
# let data =
datalit!({ 1u32, 2u32, })
# ;
```

A block appends its contents in the order provided. This can be
used for visual grouping. When a block is labeled, the bounds of the label
span from before the block to after the block.

## Simple arrays

```rust
# use datalit::datalit;
# let data =
datalit!([ 0u8; 100 ])
# ;
```

Simple arrays of the form `[ entry; N ]` will repeat the entry exactly `N`
times. N must be an unsuffixed integer literal (underscores allowed).

## Compound arrays

```rust
# use datalit::datalit;
# let data =
datalit!([{ 1u8, 2u32 }; 20])
# ;
```

Repeats its contents like simple arrays, but allows any number of entries
within the braces. While expressions from within the array can reference
labels, no labels can be defined inside of the braces.

## Align

```rust
# use datalit::datalit;
# let data =
datalit!(0xAA, align(4))
# ;
```

Aligns the current data offset to the next multiple of the given power of two.
If already aligned, nothing is appended. Padding bytes are `0x00`. A non–power-
of–two argument causes a compile error.

## Mode changes

```rust
# use datalit::datalit;
# let _ =
datalit!(
  1u32,          // native (depends on target)
  @endian = le,
  1u32,          // bytes: 01 00 00 00
  @endian = be,
  1u32,          // bytes: 00 00 00 01
  @endian = ne,
  1u32,          // native again
)
# ;
```

Mode changes adjust defaults (currently only integer endianness). The initial
endian mode is native (`ne`). It persists until changed again.

## Expression Entries

```rust
# use datalit::datalit;
# let _ =
datalit!(
  start('label): u32,
  'label: 0xFEEDF035,
)
# ;
```

Expression entries append values computed from an expression. Expression
entries must declare their output type so the macro can predict how many bytes
will be appended and how to format the value.

If an expression creates a value that is not representable by the given type,
it will generate a compilation error.

For the different expressions available, see the Expressions section below.

# Entry Sequences

In both the body of the top-level macro, as well as blocks, entries are
separated by commas. Commas are required between any two entries. Trailing
commas are permitted.

# Expressions

These are the currently available expressions:

## Start Offset

```ignore
start('label)
```

Returns the unsigned byte offset of the start of the labeled entry from the
beginning of the returned byte slice.

## End Offset

```ignore
end('label)
```

Returns the unsigned byte offset of the end of the labeled entry from the
beginning of the returned byte slice.

## Entry Length

```ignore
len('label)
```

Returns the length of the labeled entry in bytes (i.e. `end('label) - start('label)`).

# Errors

`datalit!()` has to be sure that the data it generates is unambiguous. To do
this, we enforce the following rules, aside from ensuring the syntax is correct.
All errors reference which of the entries caused the error to occur.

## Typed integers must fit within their type

We do not allow a number whose value falls outside the representable range of
the annotated type. The following is invalid:

```compile_fail
# use datalit::datalit;
# let _ =
datalit!(512u8)
# ;
```

This also applies to expression entries. If the generated value is too large
to fit in the target type, the entry causes a compile error:

```compile_fail
# use datalit::datalit;
# let _ =
datalit!(
  start('big_offset): u8,
  // A "lot" of data
  [ 0x00; 500 ],
  'big_offset: 0x77,
)
# ;
```

## Labels cannot be defined more than once

You cannot reuse a label name:

```compile_fail
# use datalit::datalit;
# let _ =
datalit!(
  start('label): u32,
  'label: 89u8,
  'label: 44u8,
)
# ;
```

## Referenced labels must exist

You cannot reference a label that has no associated entry (forward references
are allowed; the label must eventually be defined):

```compile_fail
# use datalit::datalit;
# let data =
datalit!(
  start('label): u32,
)
# ;
```

## Labels are forbidden in arrays

You cannot use a label inside an array expression, either simple or compound:

```compile_fail
# use datalit::datalit;
# let data =
datalit!(
  [{
    'label: 0xAAAA
  }; 10]
)
# ;
```

This restriction may be loosened in the future if reasonable scoping semantics
for array labels are defined.

# Guarantees

- **Fully const**: The generated data is entirely produced at compile time, and
  the resulting values are usable in const contexts.
- **Deterministic**: The generation process ensures that the generated data is
  identical from run to run.
- **`no_std`-compatible**: The generated byte slice is static and does not
  depend on an allocator.
