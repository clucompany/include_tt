//Copyright 2023-2025 #UlinProject Denis Kotlyarov (Денис Котляров)

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

/*! Macro for embedding (trees, strings, arrays) into macro trees directly from files.
```rust
use include_tt::inject;
use std::fmt::Write;

// Example demonstrating the usage of inject! macro for embedding content from files.
{
	// Embedding trees from a file in an arbitrary place of other macros.
	let a = 10;
	let b = 20;
	let mut end_str = String::new();

	// Using inject! to embed content into a macro.
	inject! {
		let _e = write!(
			&mut end_str,

			"arg1: {}, arg2: {}",

			// This file contains `a, b`.
			#include!("./examples/full.tt") // this file contains `a, b`.
		);
	}

	// Asserting the result matches the expected output.
	assert_eq!(end_str, "arg1: 10, arg2: 20");
}

{
	// Loading a string from "full.tt" using inject! macro.
	let str = inject!(
		#include_str!("./examples/full.tt") // this file contains `a, b`.
	);

	// Asserting the result matches the expected output.
	assert_eq!(str, "a, b");
}

{
	// Loading a array from "full.tt" using inject! macro.
	let array: &'static [u8; 4] = inject!(
		#include_arr!("./examples/full.tt") // this file contains `a, b`.
	);

	// Asserting the result matches the expected output.
	assert_eq!(array, b"a, b");
}
```
*/

// #![no_std] TODO, impossible without: [std::io::Error, std::{io::Read, fs::File}, std::fs::read_to_string]
#![allow(clippy::tabs_in_doc_comments)]

extern crate alloc;
extern crate proc_macro;

use crate::trees::null::make_null_group;
use crate::trees::throw_sg_err;
use crate::{
	include::{
		IncludeArr, IncludeStr, IncludeTT, IncludeTTAndFixUnkStartToken, macro_rule_include,
	},
	trees::{
		null::make_null_ttree,
		replace::{replace_tree_in_group, replace_tree_in_stream},
		result::TreeResult,
		search::SearchGroup,
		tq,
	},
};
use core::slice::IterMut;
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Span, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::{format_ident, quote};
use std::path::Path;

/// Components, templates, code for the search
/// and final construction of trees.
pub(crate) mod trees {
	pub mod group;
	pub mod null;
	pub mod replace;
	pub mod search;

	#[macro_use]
	pub mod result;
	#[allow(clippy::single_component_path_imports)]
	pub(crate) use tq;

	#[macro_use]
	pub mod sq_err;
	#[allow(clippy::single_component_path_imports)]
	pub(crate) use throw_sg_err;
	pub mod loader;
}

/// Separate syntactic expressions of trees.
pub(crate) mod exprs {
	pub mod literal;
}

/// Code component of macros.
pub(crate) mod include;

pub(crate) struct PointTrack<'tk> {
	prefix_token: &'tk mut TokenTree2,
	name_token: &'tk mut TokenTree2,
	data_token: &'tk mut TokenTree2,
	appends_files: usize,
	globalposnum: usize,
}

impl<'tk> PointTrack<'tk> {
	#[inline]
	pub const fn new(
		globalposnum: usize, 
		prefix_token: &'tk mut TokenTree2,
		name_token: &'tk mut TokenTree2,
		data_token: &'tk mut TokenTree2,
	) -> Self {
		Self {
			prefix_token,
			name_token,
			data_token,
			appends_files: 0,
			globalposnum,
		}
	}
	
	#[inline]
	pub fn prefix_span(&self) -> Span {
		self.prefix_token.span()
	}
	
	#[inline]
	pub fn name_span(&self) -> Span {
		self.name_token.span()
	}
	
	#[inline]
	pub fn data_span(&self) -> Span {
		self.data_token.span()
	}
	
	#[inline]
	pub const fn is_rewritten(&self) -> bool {
		self.appends_files > 0
	}

	pub fn into_token_tree2(self) -> Option<(usize, TokenTree2)> {
		match self.appends_files {
			0 => None,
			appends_files => {
				let data_span = self.data_span();

				Some((
					appends_files,
					std::mem::replace(self.data_token, make_null_group(data_span)),
				))
			}
		}
	}

	pub fn append_track_file(&mut self, path: &Path) {
		let name_const = format_ident!(
			"_TRACKER_FILE_NUM_{}",
			self.globalposnum + self.appends_files
		);

		let path = format!("../{}", path.display());
		let ts2 = TokenStream2::from_iter(quote! {
			/// This is a file tracker point, automatically generated by `#POINT_TRACKER_FILES;`
			const #name_const: &'static [u8] = include_bytes!(#path) as &[_];
		});

		self.append_track_files_ts(ts2)
	}

	pub fn append_track_files_ts(&mut self, ts2: TokenStream2) {
		let data_span = self.data_span();
		let is_initappendfiles = self.appends_files == 0;
		self.appends_files += 1;

		let mut ngroup = Group::new(Delimiter::None, ts2);
		ngroup.set_span(data_span);

		if is_initappendfiles {
			*self.data_token = ngroup.into();
		} else {
			match &mut self.data_token {
				TokenTree2::Group(group) => {
					let mut new_group: Vec<TokenTree2> = group.stream().into_iter().collect();
					new_group.push(ngroup.into());

					let mut ngroup =
						Group::new(Delimiter::None, TokenStream2::from_iter(new_group));
					ngroup.set_span(data_span);

					*self.data_token = ngroup.into();
				}
				_ => panic!(
					"Undefined behavior reported in `PointTrack`, someone redefined `TokenTree2`, expected `TokenTree2::Group`"
				),
			}
		}
	}
}

impl<'tk> Drop for PointTrack<'tk> {
	fn drop(&mut self) {
		if !self.is_rewritten() {
			let data_span = self.data_span();
			*self.data_token = make_null_group(data_span);
		}
		*self.prefix_token = make_null_group(self.prefix_token.span());
		*self.name_token = make_null_group(self.prefix_token.span());
	}
}

/// The task of the function is to find a group with the desired macro
/// and perform useful work specific to the selected macro.
///
/// The design of this feature has been adapted to search for attachments.
fn search_include_and_replacegroup<'tk, 'gpsn>(
	globalposnum: &'gpsn mut usize,
	mut iter: IterMut<'tk, TokenTree2>,
	point_track_file: &'_ mut Option<PointTrack<'tk>>,
) -> SearchGroup {
	'sbegin: while let Some(m_punct) = iter.next() {
		match m_punct {
			TokenTree2::Punct(punct) if punct.as_char() == '#' => {
				if let Some(m_ident) = iter.next() {
					if let TokenTree2::Ident(ident) = m_ident {
						#[allow(clippy::type_complexity)]
						let (is_add_auto_break, macro_fn): (
							bool,
							fn(&Group, Option<&mut PointTrack<'tk>>) -> TreeResult<TokenTree2>,
						) = {
							match ident {
								ident if ident == "AS_IS" => {
									/*
										Stop indexing after the given keyword. This saves resources.
									*/
									if let Some(m_punct2) = iter.next() {
										#[allow(clippy::collapsible_match)]
										if let TokenTree2::Punct(punct2) = m_punct2 {
											if punct2.as_char() == ':' {
												let nulltt = make_null_ttree();

												*m_ident = nulltt.clone();
												*m_punct = nulltt.clone();
												*m_punct2 = nulltt;
												
												return SearchGroup::Break;
											}
										}
									}

									throw_sg_err! {
										return [ident.span()]: "`:` was expected."
									}
								}
								ident if ident == "POINT_TRACKER_FILES" => {
									if let Some(m_punct2) = iter.next() {
										if let TokenTree2::Punct(punct2) = m_punct2 {
											if punct2.as_char() == ':' {
												*point_track_file =
													Some(PointTrack::new(*globalposnum, m_punct, m_ident, m_punct2));

												continue 'sbegin;
											}
										}
									}

									throw_sg_err! {
										return [ident.span()]: "`:` was expected."
									}
								}
								ident if ident == "tt" => {
									(false, macro_rule_include::<IncludeTT>)
								}

								ident if ident == "ctt" => {
									(false, macro_rule_include::<IncludeTTAndFixUnkStartToken>)
								}

								ident if ident == "str" => {
									(false, macro_rule_include::<IncludeStr>)
								}
								ident if ident == "arr" || ident == "array" => {
									(false, macro_rule_include::<IncludeArr>)
								}

								ident if ident == "break" || ident == "BREAK" => {
									/*
										Stop indexing after the given keyword. This saves resources.
									*/
									if let Some(m_punct2) = iter.next() {
										if let TokenTree2::Punct(punct2) = m_punct2 {
											if punct2.as_char() == ';' {
												let nulltt = make_null_ttree();

												*m_ident = nulltt.clone();
												*m_punct = nulltt.clone();
												*m_punct2 = nulltt;

												return SearchGroup::Break;
											}
										}
									}

									throw_sg_err! {
										return [ident.span()]: "`;` was expected."
									}
								}

								_ => throw_sg_err! {
									return [ident.span()]: "Unknown macro, expected `include`, `include_tt`, `include_and_fix_unknown_start_token`, `include_tt_and_fix_unknown_start_token`, `include_str`, `include_arr`, `include_and_break`, `include_tt_and_break`, `include_and_fix_unknown_start_token_and_break`, `include_tt_and_fix_unknown_start_token_and_break`, `include_str_and_break`, `include_arr_and_break`."
								},
							}
						};

						if let Some(m_punct2) = iter.next() {
							if let TokenTree2::Punct(punct2) = m_punct2 {
								if punct2.as_char() == '!' {
									if let Some(m_group) = iter.next() {
										if let TokenTree2::Group(group) = m_group {
											let result =
												tq!(macro_fn(group, point_track_file.as_mut()));

											let nulltt = make_null_ttree();

											*m_ident = nulltt.clone();
											*m_punct = nulltt.clone();
											*m_punct2 = nulltt.clone();
											*m_group = result;

											match is_add_auto_break {
												false => continue 'sbegin,
												true => return SearchGroup::Break,
											}
										}
									}
								}
							}
						}
					}
				}
			}
			// If this is a group, then you need to go down inside the
			// group and look for the necessary macros there.
			TokenTree2::Group(group) => match replace_tree_in_group(group, |iter| {
				let mut prefixgroup;
				let mut namegroup;
				let mut datagroup;
				#[allow(clippy::manual_map)] // see ngroup
				let mut ptf = match point_track_file {
					Some(point_track_file) => Some({
						prefixgroup = make_null_group(point_track_file.prefix_span());
						namegroup = make_null_group(point_track_file.name_span());
						datagroup = make_null_group(point_track_file.data_span());

						PointTrack::new(*globalposnum, &mut prefixgroup, &mut namegroup, &mut datagroup)
					}),
					None => None,
				};

				let result = search_include_and_replacegroup(globalposnum, iter, &mut ptf);
				if let Some(ptf) = ptf {
					if ptf.is_rewritten() {
						if let Some(point_track_file) = point_track_file {
							match ptf.into_token_tree2() {
								Some((appends_files, TokenTree2::Group(group))) => {
									*globalposnum += appends_files;
									
									point_track_file.append_track_files_ts(group.stream());
								}
								_ => panic!(
									"Undefined behavior reported in `PointTrack`, someone redefined `TokenTree2`, expected `TokenTree2::Group`"
								),
							}
						}
					}
				}
				result
			}) {
				SearchGroup::Break => continue 'sbegin,
				result @ SearchGroup::Error(..) => return result,
			},
			_ => {}
		}
	}

	SearchGroup::Break
}

/// Macro for including trees, strings, arrays from files.
///
/// Multiple occurrences of groups are supported.
///
/// ```rust
/// use include_tt::inject;
/// use std::fmt::Write;
///
/// { // Embedding compiler trees from a file in an arbitrary place of other macros.
///		let a = 10;
///		let b = 20;
///
///		let mut end_str = String::new();
///		inject! {
///			let _e = write!(
///				&mut end_str,
///
///				"arg1: {}, arg2: {}",
///				#include!("./examples/full.tt") // this file contains `a, b`.
///			);
///		}
///		assert_eq!(end_str, "arg1: 10, arg2: 20");
///	}
///
/// {
///		let str = inject!(
///			#include_str!("./examples/full.tt") // this file contains `a, b`.
///		);
///		assert_eq!(str, "a, b");
///	}
///
///	{
///		let array: &'static [u8; 4] = inject!(
///			#include_arr!("./examples/full.tt") // this file contains `a, b`.
///		);
///		assert_eq!(array, b"a, b");
///	}
/// ```
#[proc_macro]
pub fn inject(input: TokenStream) -> TokenStream {
	let mut tt: TokenStream2 = input.into();

	match replace_tree_in_stream(&mut tt, |iter| {
		search_include_and_replacegroup(&mut 0, iter, &mut None)
	}) {
		SearchGroup::Error(e) => e.into(),
		SearchGroup::Break => tt.into(),
	}
}
