# hel-random

Simple RNG with weak source of entropy(alloc) and xoshiro256++ hashing

## Examples

```rust
use hel_random::u64;

let a: u64 = u64();
let b: u64 = u64();

assert!(a != b);
```

## How to install

```
cargo add hel-random
```
