use include_tt::inject;

macro_rules! test_rules {
	[
		$a:ident + $b:ident = $c:ident

		$($tt:tt)*
	] => {
		let $c = $a + $b;

		test_rules! {
			$($tt)*
		}
	};
	[
		$a:ident - $b:ident = $c:ident

		$($tt:tt)*
	] => {
		let $c = $a - $b;

		test_rules! {
			$($tt)*
		}
	};
	[ {$($tt:tt)*} ] => {
		test_rules! {
			$($tt)*
		}
	};
	[] => []
}

fn main() {
	let a = 10;
	let b = 20;

	inject! {
		// this macro only supports:
		// a + b = n
		// or
		// a - b = n
		#POINT_TRACKER_FILES;
		test_rules! {
			#include!("./examples/mrules.tt") // this file contains "a + b = n", see "./for_examples/mrules.tt"
		}
	}
	assert_eq!(n, a + b);
	println!("n: {n:?}"); // 30
}
