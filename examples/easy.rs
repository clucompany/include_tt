
use include_tt::include_tt;

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

	include_tt! {
		test_rules! {
			#include!("./for_examples/easy.tt")
		}
	}
	assert_eq!(n, a+b);
	println!("n: {:?}", n); // << n :) $_$
}
