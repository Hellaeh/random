#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn u64_bench(b: &mut Bencher) {
	b.iter(hel_random::u64);
	assert!(hel_random::u64() > 0);
}
