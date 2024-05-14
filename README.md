<div id="header" align="center">

  <b>[include_tt]</b>
  
  (Macro for embedding (trees, strings, arrays) into macro trees directly from files.)
  </br></br>

<div id="badges">
  <a href="./LICENSE_MIT">
    <img src="https://github.com/UlinProject/img/blob/main/short_32/mit.png?raw=true" alt="mit"/>
  </a>
  <a href="./LICENSE_APACHE">
    <img src="https://github.com/UlinProject/img/blob/main/short_32/apache2.png?raw=true" alt="apache2"/>
  </a>
  <a href="https://crates.io/crates/include_tt">
    <img src="https://github.com/UlinProject/img/blob/main/short_32/cratesio.png?raw=true" alt="cratesio"/>
  </a>
  <a href="https://docs.rs/include_tt">
    <img src="https://github.com/UlinProject/img/blob/main/short_32/docrs.png?raw=true" alt="docrs"/>
  </a>
  <a href="https://github.com/denisandroid">
    <img src="https://github.com/UlinProject/img/blob/main/short_32/uproject.png?raw=true" alt="uproject"/>
  </a>
  <a href="https://github.com/clucompany">
    <img src="https://github.com/UlinProject/img/blob/main/short_32/clulab.png?raw=true" alt="clulab"/>
  </a>
	
  [![CI](https://github.com/clucompany/include_tt/actions/workflows/CI.yml/badge.svg?event=push)](https://github.com/clucompany/include_tt/actions/workflows/CI.yml) 


</div>
</div>

## Usage:

Add this to your Cargo.toml:

```toml
[dependencies]
include_tt = "1.0.4"
```

and this to your source code:
```rust
use include_tt::include_tt;
```

## Example:

```rust
use include_tt::include_tt;
use std::fmt::Write;

// Example demonstrating the usage of include_tt! macro for embedding content from files.
{ 
	// Embedding trees from a file in an arbitrary place of other macros.
	let a = 10;
	let b = 20;
	let mut end_str = String::new();
	
	// Using include_tt! to embed content into a macro.
	include_tt! {
		let _e = write!(
			&mut end_str,
			
			"arg1: {}, arg2: {}",
			#include!("./for_examples/full.tt") // this file contains `a, b`.
		);
	}
	
	// Asserting the result matches the expected output.
	assert_eq!(end_str, "arg1: 10, arg2: 20");
}

{ 
	// Loading a string from "full.tt" using include_tt! macro.
	let str = include_tt!(
		#include_str!("./for_examples/full.tt") // this file contains `a, b`.
	);
	
	// Asserting the result matches the expected output.
	assert_eq!(str, "a, b");
}

{
	// Loading a array from "full.tt" using include_tt! macro.
	let array: &'static [u8; 4] = include_tt!(
		#include_arr!("./for_examples/full.tt") // this file contains `a, b`.
	);
	
	// Asserting the result matches the expected output.
	assert_eq!(array, b"a, b");
}
```

<a href="./examples">
  See all
</a>

## License:
This project has a dual license according to (LICENSE-MIT) and (LICENSE-APACHE-2-0).

<div align="left">
  <a href="https://github.com/denisandroid">
    <img align="left" src="https://github.com/UlinProject/img/blob/main/block_220_100/uproject.png?raw=true" alt="uproject"/>
  </a>
  <b>&nbsp;Copyright (c) 2023-2024 #UlinProject</b>
	
  <b>&nbsp;(Denis Kotlyarov).</b>
  </br></br></br>
</div>

### Apache License:
<div align="left">
  <a href="./LICENSE_APACHE">
    <img align="left" src="https://github.com/UlinProject/img/blob/main/block_220_100/apache2.png?raw=true" alt="apache2"/>
    
  </a>
  <b>&nbsp;Licensed under the Apache License, Version 2.0.</b>
  </br></br></br></br>
</div>

### MIT License:
<div align="left">
  <a href="./LICENSE_MIT">
    <img align="left" src="https://github.com/UlinProject/img/blob/main/block_220_100/mit.png?raw=true" alt="mit"/>
  </a>
  <b>&nbsp;Licensed under the MIT License.</b>
  </br></br></br></br>
</div>
