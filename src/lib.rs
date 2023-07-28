//Copyright 2023 #UlinProject Denis Kotlyarov (Денис Котляров)

//-----------------------------------------------------------------------------
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at

//	   http://www.apache.org/licenses/LICENSE-2.0

//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
// limitations under the License.
//-----------------------------------------------------------------------------

// or

//-----------------------------------------------------------------------------
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:

//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.

//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

/*! Macro for including trees, strings, arrays from files. 
```rust
use include_tt::include_tt;
use std::fmt::Write;

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

{ // Loading a string from a file.
	let str = include_tt!(
		#include_str!("./for_examples/full.tt")
	);
	assert_eq!(str, "a, b");
}

{ // Loading an array from a file.
	let array: &'static [u8; 4] = include_tt!(
		#include_arr!("./for_examples/full.tt")
	);
	assert_eq!(array, b"a, b");
}
```
*/

use std::slice::IterMut;
use proc_macro2::TokenTree as TokenTree2;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use trees::sg_err;
use crate::{trees::{null::make_null_ttree, replace::{support_replace_tree_in_group, support_replace_tree_in_stream}, result::TreeResult, ttry, search::SearchGroup}, macros::include::{macro_rule_include, IncludeTt, IncludeStr, IncludeArr, IncludeTtAndFixUnkStartToken}};

/// Components, templates, code for the search 
/// and final construction of trees.
pub (crate) mod trees {
	pub (crate) mod null;
	pub (crate) mod replace;
	pub (crate) mod group;
	pub (crate) mod search;
	
	#[macro_use]
	pub (crate) mod result;
	pub (crate) use ttry;
	
	#[macro_use]
	pub (crate) mod sq_err;
	pub (crate) use sg_err;
	pub (crate) mod loader;
}

/// Separate syntactic expressions of trees.
pub (crate) mod exprs {
	pub mod literal;
}

/// Code component of macros.
pub (crate) mod macros {
	pub mod include;
}

/// The task of the function is to find a group with the desired macro 
/// and perform useful work specific to the selected macro. 
/// 
/// The design of this feature has been adapted to search for attachments.
fn search_include_and_replacegroup(
	iter: &mut IterMut<'_, TokenTree2>,
	_is_zero_glevel: bool
) -> SearchGroup {
	while let Some(m_punct) = iter.next() {
		match m_punct {
			TokenTree2::Punct(punct) => {
				if punct.as_char() == '#' {
					if let Some(m_ident) = iter.next() {
						if let TokenTree2::Ident(ident) = m_ident {
							let macro_fn = {
								match ident.to_string().as_str() {
									"include" | "include_tt" => macro_rule_include::<IncludeTt>,
									"include_and_fix_unknown_start_token" => macro_rule_include::<IncludeTtAndFixUnkStartToken>,
									"include_str" => macro_rule_include::<IncludeStr>,
									"include_arr" => macro_rule_include::<IncludeArr>,
									
									_ => continue,
								}
							};
							
							if let Some(m_punct2) = iter.next() {
								if let TokenTree2::Punct(punct2) = m_punct2 {
									if punct2.as_char() == '!' {
										if let Some(m_group) = iter.next() {
											if let TokenTree2::Group(group) = m_group {
												let result = ttry!(macro_fn(
													group,
												));
												
												match result {
													None => {}, // skip,
													Some(new_tt) => {
														let nulltt = make_null_ttree();
													
														*m_ident = nulltt.clone();
														*m_punct = nulltt.clone();
														*m_punct2 = nulltt;
														
														*m_group = new_tt;
													},
												}
											}
										}
									}
									// autoskip
								}
							}
						}
					}
				}
			}
			// If this is a group, then you need to go down inside the 
			// group and look for the necessary macros there.
			TokenTree2::Group(group) => match support_replace_tree_in_group(
				group,
				|mut iter| search_include_and_replacegroup(&mut iter, false),
			) {
				SearchGroup::Break => continue,
				result @ SearchGroup::Error(..) => return result,
			},
			_ => {},
		}
	}
	
	SearchGroup::Break
}

/// Macro for including trees, strings, arrays from files. 
/// 
/// Multiple occurrences of groups are supported.
/// 
/// ```rust
/// use include_tt::include_tt;
/// use std::fmt::Write;
/// 
/// { // Embedding compiler trees from a file in an arbitrary place of other macros.
///		let a = 10;
///		let b = 20;
///		
///		let mut end_str = String::new();
///		include_tt! {
///			let _e = write!(
///				&mut end_str,
///				
///				"arg1: {}, arg2: {}",
///				
///				// This file contains `a, b`.
///				#include!("./for_examples/full.tt")
///			);
///		}
///		assert_eq!(end_str, "arg1: 10, arg2: 20");
///	}
/// 
/// { // Loading a string from a file.
///		let str = include_tt!(
///			#include_str!("./for_examples/full.tt")
///		);
///		assert_eq!(str, "a, b");
///	}
///	
///	{ // Loading an array from a file.
///		let array: &'static [u8; 4] = include_tt!(
///			#include_arr!("./for_examples/full.tt")
///		);
///		assert_eq!(array, b"a, b");
///	}
/// ```
#[proc_macro]
pub fn include_tt(input: TokenStream) -> TokenStream {
	let mut tt: TokenStream2 = input.into();

	match support_replace_tree_in_stream(
		&mut tt,
		|mut iter| search_include_and_replacegroup(&mut iter, true)
	) {
		SearchGroup::Error(e) => return e.into(),
		SearchGroup::Break => {},
	}
	tt.into()
}
