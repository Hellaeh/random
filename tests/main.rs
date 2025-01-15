#![feature(test)]

extern crate test;

use std::collections::HashSet;

use test::Bencher;

use hel_random::*;

#[test]
fn flip_a_coin_fairness() {
	const TRIES: i64 = 1_000_000;

	let mut balance = 0;

	bool();
	bool();
	bool();
	bool();
	bool();

	for _ in 0..TRIES {
		balance += if bool() { 1 } else { -1 };
	}

	println!("Fairness: {balance}");
	assert!(balance < (TRIES / 100));
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

// make_test!(test_f32, bench_f32, f32);
// make_test!(test_f64, bench_f64, f64);

macro_rules! make_ignored {
	($test_name: ident, $fn_name: ident) => {
		#[test]
		#[ignore = r#"to see output use "cargo t -- --ignored""#]
		fn $test_name() {
			$fn_name();

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

// make_ignored!(output_f32, f32);
// make_ignored!(output_f64, f64);

// This test can fail sometimes
#[test]
#[ignore = "this can fail sometimes due to race condition"]
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
