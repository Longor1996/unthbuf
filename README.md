# UnthBuf

> **Note:** While tested and usable, this library is not quite ready;
> expect breaking changes with any update until `1.0.0`.

The `UnthBuf` is a structure that stores a given fixed amount of unsigned integers, like `Box<[usize]>`... except that the *bit-size* can be freely chosen from `0` to `64` bits, and the alignment of the stored values is a const-generic `bool`, effectively making it a `Box<[uN]>`.

For example:

```rust
use unthbuf::UnthBuf;
let mut buf = UnthBuf::<true>::new(4096, 5);
buf.set(21, 5).unwrap();
```

Will, if `ALIGNED == true`, result in this bit-pattern:

```text
0101101101101101101101101101101101101101101101101101101101101101
0000000000000000000000000000000000000000000000000000000000000101
```

Or, if `ALIGNED == false`:

```text
1101101101101101101101101101101101101101101101101101101101101101
0000000000000000000000000000000000000000000000000000000000000010
```
