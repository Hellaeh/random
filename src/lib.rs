//! A simple pseudo non-cryptographic random number generator.
//! Using xoshiro256++ under the hood.
#![deny(missing_docs)]
//
#![feature(test)]
#![feature(thread_local)]
#![feature(const_alloc_layout)]

use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Once,
};

const LOCAL_STATE_SIZE: usize = 4;

type Target = u64;
type LocalStateType = [Target; LOCAL_STATE_SIZE];

// Will be used as a multiplier for thread local rng
static mut SHARED_STATE: AtomicUsize = AtomicUsize::new(1);
#[thread_local]
// Will be used as a primal rng state
static mut LOCAL_STATE: LocalStateType = [0, 0, 0, 0];
// TODO remove this
#[thread_local]
static mut INIT: Once = Once::new();

fn init() {
	unsafe {
		use std::alloc::*;

		let seed = SHARED_STATE.fetch_add(1, Ordering::Relaxed) as Target;

		let mut res = LOCAL_STATE;

		const ALLOC: usize = LOCAL_STATE_SIZE * LOCAL_STATE_SIZE;

		let layout = Layout::array::<Target>(ALLOC).unwrap();
		let ptr = alloc(layout);

		if ptr.is_null() {
			handle_alloc_error(layout);
		}

		let garbage_arr = &mut *(ptr as *mut [Target; ALLOC]);

		let addr = ptr as Target;
		// Also use pointer itself as value
		let mut bits = addr ^ (addr >> 11) ^ (addr.rotate_right(30));
		// Looking for garbage on the heap, while writing some garbage back
		for (i, trash) in garbage_arr.iter_mut().enumerate() {
			let current = &mut res[i % LOCAL_STATE_SIZE];

			let val = match *trash {
				0 => {
					let msb = (bits & 1) ^ ((bits >> 1) & 1);
					bits >>= 1;
					bits |= msb << 15;
					bits
				}
				n => n,
			};

			*current = (*current ^ val).wrapping_mul(seed);

			*trash = val
		}

		LOCAL_STATE = res;

		dealloc(ptr, layout)
	}
}

macro_rules! make {
	($type: ident, $rest: block) => {
		#[doc = concat!("Will generate a random", stringify!($type))]
		///
		/// # Example
		/// ```
		#[doc = concat!("let a: ", stringify!($type), " = hel_random::", stringify!($type), "();")]
		#[doc = concat!("let b: ", stringify!($type), " = hel_random::", stringify!($type), "();")]
		/// // Examples generated by macro, so we check if type is `bool` or `i8` or `u8` to avoid collisions
		#[doc = concat!("if std::mem::size_of::<", stringify!($type), ">() > 1 {")]
		///     assert!(a != b);
		/// }
		/// ```
		#[inline]
		pub fn $type() -> $type {
			unsafe {
				// this adds 2x performance loss
				// TODO remove it: make user call it explicitly or 'life before main'
				INIT.call_once(init);

				$rest
			}
		}
	};

	($type: ident) => {
		make!($type, { u64() as $type });
	};
}

make!(u64, {
	let res = LOCAL_STATE[0]
		.wrapping_add(LOCAL_STATE[3])
		.rotate_left(23)
		.wrapping_add(LOCAL_STATE[0]);

	let s = LOCAL_STATE[1] << 17;

	LOCAL_STATE[2] ^= LOCAL_STATE[0];
	LOCAL_STATE[3] ^= LOCAL_STATE[1];
	LOCAL_STATE[1] ^= LOCAL_STATE[2];
	LOCAL_STATE[0] ^= LOCAL_STATE[3];

	LOCAL_STATE[2] ^= s;

	LOCAL_STATE[3] = LOCAL_STATE[3].rotate_left(45);

	res
});
make!(u128, { u64() as u128 | ((u64() as u128) << 64) });
make!(i128, { u128() as i128 });
make!(i64);
make!(u32);
make!(i32);
make!(u16);
make!(i16);
make!(u8);
make!(i8);
make!(bool, {
	loop {
		let r = u64();

		let a = (r & 1) == 1;
		let b = (r & 2) == 2;

		if a != b {
			return a;
		}
	}
});

#[cfg(test)]
mod tests {
	extern crate test;

	use std::collections::HashSet;

	use test::Bencher;

	use super::*;

	#[test]
	fn flip_a_coin_fairness() {
		unsafe {
			const TRIES: i64 = 1_000_000;

			let mut balance = 0;

			bool();

			println!("State: {:?}", LOCAL_STATE);
			println!(
				"Population: {}",
				LOCAL_STATE.iter().fold(0, |acc, s| acc + s.count_ones())
			);

			for _ in 0..TRIES {
				balance += if bool() { 1 } else { -1 };
			}

			println!("Fairness: {balance}");
			assert!(balance < (TRIES / 100));
		}
	}

	macro_rules! make_test {
		($test_name: ident, $subject: ident) => {
			#[bench]
			fn $test_name(b: &mut Bencher) {
				b.iter($subject)
			}
		};
	}

	make_test!(test_u128, u128);
	make_test!(test_i128, i128);
	make_test!(test_u64, u64);
	make_test!(test_i64, i64);
	make_test!(test_u32, u32);
	make_test!(test_i32, i32);
	make_test!(test_u16, u16);
	make_test!(test_i16, i16);
	make_test!(test_u8, u8);
	make_test!(test_i8, i8);
	make_test!(test_bool, bool);

	macro_rules! make_ignored {
		($test_name: ident, $fn_name: ident) => {
			#[test]
			#[ignore = r#"to see output use "cargo t -- --ignored""#]
			fn $test_name() {
				$fn_name();

				unsafe {
					println!("{:?}", LOCAL_STATE);
				}

				for _ in 0..100 {
					println!("{}", $fn_name());
				}

				assert!(false);
			}
		};
	}

	make_ignored!(output_u128, u128);
	make_ignored!(output_i128, i128);
	make_ignored!(output_u64, u64);
	make_ignored!(output_i64, i64);
	make_ignored!(output_u32, u32);
	make_ignored!(output_i32, i32);
	make_ignored!(output_u16, u16);
	make_ignored!(output_i16, i16);
	make_ignored!(output_u8, u8);
	make_ignored!(output_i8, i8);
	make_ignored!(output_bool, bool);

	#[test]
	fn multithreaded() {
		const THREADS: usize = 128;
		let mut threads = Vec::new();

		for _ in 0..THREADS {
			threads.push(std::thread::spawn(u64))
		}

		let mut res: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
		res.sort();

		println!("{:?}", &res);

		let set: HashSet<_> = HashSet::from_iter(res);

		println!("{:?}", &set);

		assert_eq!(set.len(), THREADS);
	}
}