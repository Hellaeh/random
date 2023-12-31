//! A simple pseudo non-cryptographic random number generator.
//! Using xoshiro256++ under the hood.
#![warn(missing_docs)]
//
#![feature(test)]

const STATE_SIZE: usize = 4;

type Target = u64;
type StateType = [Target; STATE_SIZE];

static mut STATE: StateType = [0, 0, 0, 0];

#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
static INIT: extern "C" fn() = {
	extern "C" fn init() {
		unsafe {
			use std::alloc::*;

			let mut res = STATE;

			const ALLOC: usize = STATE_SIZE * STATE_SIZE;

			let layout = Layout::array::<Target>(ALLOC).unwrap();
			let ptr = alloc(layout);

			if ptr.is_null() {
				handle_alloc_error(layout);
			}

			let garbage_arr = &mut *(ptr as *mut [Target; ALLOC]);

			// Will be used if there's no garbage on the heap
			let addr = std::hint::black_box(ptr as Target);
			let mut bits = addr ^ (addr >> 11) ^ (addr.rotate_right(30));

			// Looking for garbage on the heap, while writing some garbage back
			for (i, garbage) in garbage_arr.iter_mut().enumerate() {
				let current = &mut res[i % STATE_SIZE];

				let val = std::hint::black_box(match *garbage {
					0 => {
						let msb = ((bits & 1) ^ ((bits >> 1) & 1)) << (Target::BITS - 1);
						bits >>= 1;
						bits |= msb;
						bits
					}
					n => n,
				});

				*current ^= val;
				*garbage = val
			}

			STATE = res;

			dealloc(ptr, layout)
		}
	}

	init
};

#[inline]
fn xoshiro256pp() {
	unsafe {
		let s = STATE[1] << 17;

		STATE[2] ^= STATE[0];
		STATE[3] ^= STATE[1];
		STATE[1] ^= STATE[2];
		STATE[0] ^= STATE[3];

		STATE[2] ^= s;

		STATE[3] = STATE[3].rotate_left(45);
	}
}

/// A helper trait to generate random values
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub trait Random: Sized {
	/// Will generate a random [`Self`]
	fn random() -> Self;
}

/// Generic function that returns a random [`T`]
///
/// # Example
/// ```
/// use hel_random::generate;
///
/// let a: u64 = generate();
/// let b: u32 = generate();
/// let c = generate::<i128>();
///
/// println!("a = {a}");
/// println!("b = {b}");
/// println!("c = {c}");
/// ```
#[inline(always)]
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn generate<T: Random>() -> T {
	T::random()
}

macro_rules! make {
	($type: ident, $code: block) => {
		#[doc = concat!("Will generate a random ", stringify!($type))]
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
		#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
		pub fn $type() -> $type {
			$code
		}

		impl Random for $type {
			#[doc = concat!("Will generate a random ", stringify!($type))]
			///
			/// # Example
			/// ```
			/// use hel_random::Random;
			///
			#[doc = concat!("let r = ", stringify!($type), "::random();")]
			/// println!("r = {r}");
			/// ```
			#[inline(always)]
			fn random() -> Self {
				$type()
			}
		}
	};

	($type: ident) => {
		make!($type, { u64() as $type });
	};
}

make!(u128, {
	xoshiro256pp();

	unsafe {
		STATE[0].wrapping_add(STATE[2]) as u128 | (((STATE[1]).wrapping_add(STATE[3]) as u128) << 64)
	}
});
make!(i128, { u128() as i128 });

make!(u64, {
	xoshiro256pp();
	unsafe {
		STATE[0]
			.wrapping_add(STATE[3])
			.rotate_left(23)
			.wrapping_add(STATE[0])
	}
});
make!(i64);
make!(u32);
make!(i32);
make!(u16);
make!(i16);
make!(u8);
make!(i8);

make!(bool, {
	unsafe {
		// runtime check is necessary to avoid infinite loop
		if STATE[0] == 0 {
			return false;
		}

		loop {
			xoshiro256pp();

			let a = (STATE[0] & 1) == 1;
			let b = (STATE[2] & 1) == 1;

			if a != b {
				return a;
			}
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

			println!("State: {:?}", STATE);
			println!(
				"Population: {}",
				STATE.iter().fold(0, |acc, s| acc + s.count_ones())
			);

			for _ in 0..TRIES {
				balance += if bool() { 1 } else { -1 };
			}

			println!("Fairness: {balance}");
			assert!(balance < (TRIES / 100));
		}
	}

	macro_rules! make_test {
		($test_name: ident, $bench_name: ident, $subject: ident) => {
			#[bench]
			fn $test_name(b: &mut Bencher) {
				let _w = b.iter($subject);

				assert!(u64() > 0);
			}

			#[bench]
			#[ignore = "generating 1_000_000 result. Use 'cargo bench -- --ignored'"]
			fn $bench_name(b: &mut Bencher) {
				let _w = b.iter(|| {
					let mut res = $subject();

					for _ in 0..1_000_000 {
						res = $subject();
					}

					res
				});

				assert!(u64() > 0);
			}
		};
	}

	make_test!(test_u128, bench_u128, u128);
	make_test!(test_i128, bench_i128, i128);
	make_test!(test_u64, bench_u64, u64);
	make_test!(test_i64, bench_i64, i64);
	make_test!(test_u32, bench_u32, u32);
	make_test!(test_i32, bench_i32, i32);
	make_test!(test_u16, bench_u16, u16);
	make_test!(test_i16, bench_i16, i16);
	make_test!(test_u8, bench_u8, u8);
	make_test!(test_i8, bench_i8, i8);
	make_test!(test_bool, bench_bool, bool);

	macro_rules! make_ignored {
		($test_name: ident, $fn_name: ident) => {
			#[test]
			#[ignore = r#"to see output use "cargo t -- --ignored""#]
			fn $test_name() {
				$fn_name();

				unsafe {
					println!("{:?}", STATE);
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
		const THREADS: usize = 1024;
		let mut threads = Vec::new();

		for _ in 0..THREADS {
			threads.push(std::thread::spawn(|| {
				let mut res = u64();

				for _ in 0..1_000 {
					res = u64();
				}

				res
			}))
		}

		let mut res: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
		res.sort();

		println!("{:?}", &res);

		let set: HashSet<_> = HashSet::from_iter(res);

		println!("{:?}", &set);

		assert_eq!(set.len(), THREADS);

		// assert!(false);
	}
}
