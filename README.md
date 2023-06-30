# hel-random

Simple RNG with weak entropy source (alloc) and xoshiro256** hashing

## Examples

```rust
let a: u64 = u64();
let b: u64 = u64();

assert!(a != b);
```

## How to install

```
cargo add hel-random
```
