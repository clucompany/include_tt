
use std::fmt::Write;
use include_tt::*;

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
	
	[$($tt:tt)+] => {
		compile_error!(stringify!( $($tt)* ))
	};
	[] => []
}

fn main() {
	// Loading trees from a file and substituting them into a custom macro.
	include_tt! {
		test2_rules! {
			[#include!("./for_examples/full.tt")]
			[#include! { "./for_examples/full.tt"}]
		}
		test2_rules! {
			#include!("./for_examples/full.tt")
		}
		
		println!(
			concat!(
				"#",
				#include_str!("./for_examples/full.tt"),
				"#"
			)
		);
	}
	
	{ // Loading a string from a file.
		let str = include_tt!(#include_str!("./for_examples/full.tt"));
		assert_eq!(str, "a, b");
	}
	
	{ // Loading an array from a file.
		let array: &'static [u8; 4] = include_tt!(
			#include_arr!("./for_examples/full.tt")
		);
		assert_eq!(array, b"a, b");
	}
	
	{ // Embedding compiler trees from a file in an arbitrary place of other macros.
		let a = 10;
		let b = 20;
		
		let mut end_str = String::new();
		include_tt! {
			let _e = write!(
				&mut end_str,
				
				"arg1: {}, arg2: {}",
				
				// This file contains `a, b`.
				#include!("./for_examples/full.tt")
			);
		}
		assert_eq!(end_str, "arg1: 10, arg2: 20");
	}
}
