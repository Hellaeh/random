#![feature(test)]

extern crate test;

use std::collections::HashMap;

use test::Bencher;

use hel_random::*;

#[test]
fn flip_a_coin_fairness() {
	const TRIES: i64 = 1_000_000;

	let mut balance: i64 = 0;

	bool();
	bool();
	bool();
	bool();
	bool();

	for _ in 0..TRIES {
		balance += if bool() { 1 } else { -1 };
	}

	for i in 0..10 {
		let trace = bool();
		balance += if trace { 1 } else { -1 };
		println!("Trace {}: {trace}", i + 1);
	}

	println!("Fairness: {balance}");
	assert!(balance.abs() < (TRIES / 100));
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

#[test]
#[ignore = "race conditions"]
fn multithreaded() {
	const THREADS: usize = 1024;
	let mut threads = Vec::new();

	println!("State:");
	for inner in get_state() {
		println!("{inner:066b\n}");
	}
	println!();
	// println!("Current state: {:#066b?}", get_state());

	for _ in 0..THREADS {
		threads.push(std::thread::spawn(|| {
			std::thread::park();

			let mut res = u64();

			for _ in 0..10_000 {
				res = u64();
			}

			res
		}))
	}

	// manual unroll cuz why not
	for chunk in threads.chunks(16) {
		chunk[0].thread().unpark();
		chunk[1].thread().unpark();
		chunk[2].thread().unpark();
		chunk[3].thread().unpark();
		chunk[4].thread().unpark();
		chunk[5].thread().unpark();
		chunk[6].thread().unpark();
		chunk[7].thread().unpark();
		chunk[8].thread().unpark();
		chunk[9].thread().unpark();
		chunk[10].thread().unpark();
		chunk[11].thread().unpark();
		chunk[12].thread().unpark();
		chunk[13].thread().unpark();
		chunk[14].thread().unpark();
		chunk[15].thread().unpark();
	}

	let res: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
	let mut map = HashMap::with_capacity(res.len());
	let mut collision_count = 0;

	for n in res {
		match map.get_mut(&n) {
			Some(v) => {
				collision_count += 1;
				*v += 1;
			}
			None => {
				map.insert(n, 1);
			}
		}
	}

	for (k, v) in map.iter().filter(|(_, &v)| v > 1) {
		println!(r#"Found {v} occurences of "{k}""#)
	}

	println!("Total collisions: {collision_count}");

	if collision_count == 0 {
		// passed
		return;
	}

	assert_eq!(map.len(), THREADS);

	#[allow(clippy::assertions_on_constants)]
	{
		assert!(false, "i would worry if it actually passed");
	}
}

#[test]
#[ignore]
fn average_and_deviation() {
	type Inner = u64;
	const NUM_OF_POINTS: Inner = 1 << 27;
	const MAX_VAL: f64 = Inner::MAX as f64;

	let mut tries = 0;

	let now = std::time::Instant::now();

	loop {
		tries += 1;

		let mut avg: f64 = 0.0;

		for _ in 0..NUM_OF_POINTS {
			avg += generate::<Inner>() as f64 / NUM_OF_POINTS as f64;
		}

		let deviation = ((avg - MAX_VAL / 2.0) / avg) * 100.0;

		if deviation.abs() < 0.01 && tries < 100 {
			continue;
		}

		println!("Average: {avg}");
		println!("Deviation : {deviation}");

		break;
	}

	println!("Tries: {tries}");
	println!("Elapsed: {:?}", now.elapsed());

	#[allow(clippy::assertions_on_constants)]
	{
		assert!(false, "need to fail in order to see stdout output");
	}
}
