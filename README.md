# include_tt
[![CI](https://github.com/clucompany/include_tt/actions/workflows/CI.yml/badge.svg?event=push)](https://github.com/clucompany/include_tt/actions/workflows/CI.yml)
[![Apache licensed](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/include_tt)](https://crates.io/crates/include_tt)
[![Documentation](https://docs.rs/include_tt/badge.svg)](https://docs.rs/include_tt)

Macro for including trees, strings, arrays from files. 

```rust
use include_tt::include_tt;
use std::fmt::Write;

// Substitution of a macro component from a file.
{
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

// Loading a string from a file.
{
	let str = include_tt!(
		#include_str!("./for_examples/full.tt")
	);
	assert_eq!(str, "a, b");
}

// Loading an array from a file.
{
	let array: &'static [u8; 4] = include_tt!(
		#include_arr!("./for_examples/full.tt")
	);
	assert_eq!(array, b"a, b");
}
```
