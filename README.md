# UnthBuf

The `UnthBuf` is a data-structure that holds a fixed buffer of unsigned integers, just like a `Box<[usize]>` would...
except that the *bit-size* of the integers can be adjusted from `1` to `64` bits, effectively making it a `Box<[uN]>`!

For example:

```rust
use unthbuf::{UnthBuf, Bits, aligned::AlignedLayout};
let mut buf = UnthBuf::<AlignedLayout>::new(4096, Bits::new(5).unwrap());
buf.set(21, 5).unwrap();
```

Internally the buffer is a boxed slice of `usize`d **cells**,
with the integer **elements** being stored within the cells
according to the chosen [`CellLayout`].

This will result in a bit-pattern like this:

```text
0101101101101101101101101101101101101101101101101101101101101101
0000000000000000000000000000000000000000000000000000000000000101
                            integer aligned to word boundary ^^^
```

Or, if the `PackedLayout`/[`PackedUnthBuf`] is used instead:

```text
1101101101101101101101101101101101101101101101101101101101101101
^              integer packed across word boundary            vv
0000000000000000000000000000000000000000000000000000000000000010
```

While the `PackedLayout` is certainly more compact, it is also roughly ~20% slower; use it when every bit counts.

You can use the `UnthBuf::get_padding_bit_count`-function to determine how much space is lost.
