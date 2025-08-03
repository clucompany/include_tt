use include_tt::inject;
use std::fmt::Write;

macro_rules! test2_rules {
	[
		[a, b]

		$($tt:tt)*
	] => {
		println!("test2_rules: [a, b]");
		test2_rules! {
			$($tt)*
		}
	};
	[
		[c, d]

		$($tt:tt)*
	] => {
		println!("test2_rules: [c, d]");
		test2_rules! {
			$($tt)*
		}
	};

	[
		a, b

		$($tt:tt)*
	] => {
		println!("test2_rules: a, b");
		test2_rules! {
			$($tt)*
		}
	};

	[
		$($tt:tt)+
	] => {
		compile_error!(stringify!( $($tt)* ))
	};
	[] => []
}

fn main() {
	// Loading trees from a file and substituting them into a custom macro.
	inject! {
		#POINT_TRACKER_FILES:
		test2_rules! {
			[ #include!("./examples/full.tt") ] // this file contains `a, b`.
			[ #include! { "./examples/full.tt" } ] // this file contains `a, b`.
		}
		test2_rules! {
			#include!("./examples/full.tt") // this file contains `a, b`.
		}

		println!(
			concat!(
				"#",
				#include_str!("./examples/full.tt"), // this file contains `a, b`.
				"#"
			)
		);
	}

	{
		// Loading a string from a file.
		let str = inject!(#include_str!("./examples/full.tt")); // this file contains `a, b`.
		assert_eq!(str, "a, b");
	}

	{
		// Loading an array from a file.
		let array: &'static [u8; 4] = inject!(
			#include_arr!("./examples/full.tt") // this file contains `a, b`.
		);
		assert_eq!(array, b"a, b");
	}

	{
		// Embedding compiler trees from a file in an arbitrary place of other macros.
		let a = 10;
		let b = 20;

		let mut end_str = String::new();
		inject! {
			let _e = write!(
				&mut end_str,

				"arg1: {}, arg2: {}",
				#include!("./examples/full.tt") // this file contains `a, b`.
			);
		}
		assert_eq!(end_str, "arg1: 10, arg2: 20");
	}
}
