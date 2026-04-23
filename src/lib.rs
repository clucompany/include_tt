//Copyright 2023-2026 #UlinProject Denis Kotlyarov (Денис Котляров)

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

//! # include_tt
//!
//! Macros for ultra-flexible injection of token trees, literals, or binary data
//! into Rust syntax trees from external sources.
//!
//! This crate allows you to compose file paths dynamically and include their
//! content as tokens, strings, or byte arrays directly into your source code.

/*!

```rust
use include_tt::inject;
use std::fmt::Write;
let mut buf = String::new();

inject! {
    write!(
        &mut buf,
        "Welcome, {}. Your score is {}!",
        #tt("examples/name.tt"),            // `"Ferris"`
        #tt("examples/" "score" ".tt")      // `100500`
    ).unwrap();
}

assert_eq!(buf, "Welcome, Ferris. Your score is 100500!");
```
*/

// #![no_std] TODO, impossible without: [std::io::Error, std::{io::Read, fs::File}, std::fs::read_to_string]

extern crate alloc;
extern crate proc_macro;

use crate::points::file_dep_tracker::FileDepTracker;
use crate::trees::null::make_null_group;
use crate::trees::sq_err;
use crate::{
    include::{InjectArr, InjectCTT, InjectStr, InjectTT, macro_rule_include},
    trees::{
        replace::{replace_tree_in_group, replace_tree_in_stream},
        result::TreeResult,
        search::SearchGroup,
        tq,
    },
};
use core::slice::IterMut;
use proc_macro::TokenStream;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree as TokenTree2};

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
    #[path = "sq_err.rs"]
    mod _sq_err;
    #[allow(clippy::single_component_path_imports)]
    pub(crate) use __sq_err_format;
    #[allow(clippy::single_component_path_imports)]
    pub(crate) use sq_err;
    pub mod loader;
}

/// Separate syntactic expressions of trees.
pub(crate) mod exprs {
    pub mod literal;
}

/// Code component of macros.
pub(crate) mod include;

/// Special code points that override macro behavior
pub(crate) mod points {
    pub mod file_dep_tracker;
}

/// The task of the function is to find a group with the desired macro
/// and perform useful work specific to the selected macro.
///
/// The design of this feature has been adapted to search for attachments.
fn autoinject_tt_in_group<'tk, 'gpsn>(
    globalposnum: &'gpsn mut usize,
    mut iter: IterMut<'tk, TokenTree2>,
    point_track_file: &'_ mut Option<FileDepTracker<'tk>>,
) -> SearchGroup {
    'sbegin: while let Some(mut m_punct) = iter.next() {
        'match_m_punct: loop {
            match m_punct {
                TokenTree2::Punct(punct) if punct.as_char() == '-' => {
                    // Just a way to escape `#` to prevent the macro from parsing `#` and executing it.
                    let iter_next = match iter.next() {
                        Some(a) => a,
                        None => break 'sbegin,
                    };
                    if let TokenTree2::Punct(punct) = iter_next
                        && punct.as_char() == '#'
                    {
                        *m_punct = make_null_group(m_punct.span());
                        continue 'sbegin;
                    }

                    m_punct = iter_next;
                    continue 'match_m_punct;
                }
                TokenTree2::Punct(punct) if punct.as_char() == '#' => {
                    if let Some(m_ident) = iter.next()
                        && let TokenTree2::Ident(ident) = m_ident
                    {
                        let macro_fn = match &*ident {
                            ident if ident == "AS_IS" => {
                                /*
                                    Stop indexing after the given keyword. This saves resources.
                                */
                                if let Some(m_punct2) = iter.next()
                                    && let TokenTree2::Punct(punct2) = m_punct2
                                    && punct2.as_char() == ':'
                                {
                                    *m_ident = make_null_group(m_ident.span());
                                    *m_punct = make_null_group(m_punct.span());
                                    *m_punct2 = make_null_group(m_punct2.span());

                                    return SearchGroup::Break;
                                }

                                sq_err! {
                                    return [ident.span()]: "`:` was expected."
                                }
                            }
                            ident if ident == "TRACK_FILES" || ident == "POINT_TRACKER_FILES" => {
                                if let Some(m_punct2) = iter.next()
                                    && let TokenTree2::Punct(punct2) = m_punct2
                                    && punct2.as_char() == ':'
                                {
                                    *point_track_file = Some(FileDepTracker::new(
                                        *globalposnum,
                                        m_punct,
                                        m_ident,
                                        m_punct2,
                                    ));

                                    continue 'sbegin;
                                }

                                sq_err! {
                                    return [ident.span()]: "`:` was expected."
                                }
                            }
                            ident if ident == "tt" => {
                                macro_rule_include::<InjectTT>
                                    as fn(
                                        &Group,
                                        Option<&mut FileDepTracker<'tk>>,
                                    )
                                        -> TreeResult<TokenTree2>
                            }
                            ident if ident == "ctt" => macro_rule_include::<InjectCTT> as _,
                            ident if ident == "str" => macro_rule_include::<InjectStr> as _,
                            ident if ident == "arr" || ident == "array" => {
                                macro_rule_include::<InjectArr> as _
                            }
                            ident if ident == "break" => {
                                /*
                                    Stop indexing after the given keyword. This saves resources.
                                */
                                if let Some(m_punct2) = iter.next()
                                    && let TokenTree2::Punct(punct2) = m_punct2
                                    && punct2.as_char() == ';'
                                {
                                    *m_ident = make_null_group(m_ident.span());
                                    *m_punct = make_null_group(m_punct.span());
                                    *m_punct2 = make_null_group(m_punct2.span());

                                    return SearchGroup::Break;
                                }

                                sq_err! {
                                    return [ident.span()]: "`;` was expected."
                                }
                            }

                            _ => sq_err! {
                                return [ident.span()]: "Undefined action to include data in macro or change its behavior, expected macro data type: `tt`, `ctt`, `arr`, `str`, or marker: `#AS_IS:`, `#TRACK_FILES:`, or stop parsing macro via `#break;`."
                            },
                        };

                        if let Some(m_group) = iter.next()
                            && let TokenTree2::Group(group) = m_group
                        {
                            let result = tq!(macro_fn(group, point_track_file.as_mut()));

                            *m_ident = make_null_group(m_ident.span());
                            *m_punct = make_null_group(m_punct.span());
                            *m_group = result;

                            continue 'sbegin;
                        }

                        sq_err! {
                            return [ident.span()]: "After this input, the group `()`, `[]`, `{}` is expected."
                        }
                    }
                }
                // If this is a group, then you need to go down inside the
                // group and look for the necessary macros there.
                TokenTree2::Group(group) => match replace_tree_in_group(group, |iter| {
                    let mut prefixgroup;
                    let mut namegroup;
                    let mut datagroup;
                    #[allow(clippy::manual_map)]
                    let mut ptf = match point_track_file {
                        Some(point_track_file) => Some({
                            prefixgroup = make_null_group(point_track_file.prefix_span());
                            namegroup = make_null_group(point_track_file.name_span());
                            datagroup = make_null_group(point_track_file.data_span());

                            FileDepTracker::new(
                                *globalposnum,
                                &mut prefixgroup,
                                &mut namegroup,
                                &mut datagroup,
                            )
                        }),
                        None => None,
                    };

                    let result = autoinject_tt_in_group(globalposnum, iter, &mut ptf);
                    if let Some(ptf) = ptf
                        && ptf.is_rewritten()
                        && let Some(point_track_file) = point_track_file
                    {
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
                    result
                }) {
                    SearchGroup::Break => continue 'sbegin,
                    result @ SearchGroup::Error(..) => return result,
                },
                _ => {}
            }
            break 'match_m_punct;
        }
    }

    SearchGroup::Break
}

/// Ultra-flexible macro for compile-time injection of token trees, strings, or byte arrays from files.
///
/// It scans the input for `#marker(...)` patterns, composes the file path, and replaces
/// the marker with the file's content before the Rust compiler processes the code.
///
/// ### Available Markers:
/// * `#tt(...)` — Injects content as raw Rust tokens.
/// * `#ctt(...)` — **Clean TT**: Same as `#tt`, but pre-processes the file to replace C-style
///   line continuations (`\` followed by whitespace) with spaces.
/// * `#str(...)` — Injects content as a `&'static str`.
/// * `#arr(...)` or `#array(...)` — Injects content as a `&'static [u8]`.
///
/// ### Control & Optimization:
/// * `#TRACK_FILES:` — Enables incremental compilation tracking for all included files.
/// * `#AS_IS:` or `#break;` — Stops parsing the rest of the macro input to save compile time.
/// * `-#` — Escapes the `#` symbol so it is ignored by `inject!`.
///
/// ## Example: Template-like injection
/// ```rust
/// use include_tt::inject;
/// use std::fmt::Write;
/// let mut buf = String::new();
///
/// inject! {
///     write!(
///         &mut buf,
///         "Welcome, {}. Your score is {}!",
///         #tt("examples/name.tt"),            // contents: `"Ferris"`
///         #tt("examples/" "score" ".tt")      // contents: `100500`
///     ).unwrap();
/// }
/// assert_eq!(buf, "Welcome, Ferris. Your score is 100500!");
/// ```
///
/// ## Example: Advanced Codegen & Tracking
/// ```rust
/// macro_rules! new_module {
///     [ @($const_t: ident) : [ $($path:tt)* ]; ] => {
///         include_tt::inject! {
///             #[allow(dead_code)]
///             pub mod my_module {
///                 pub const a: usize = 0;
///                 pub const b: usize = 10;
///
///                 // Track changes in external files for incremental compilation
///                 #TRACK_FILES:
///
///                 pub const $const_t: (usize, usize) = (#tt($($path)*));
///             }
///         }
///     };
/// }
///
/// new_module! {
///    @(T): [examples / "full" . t 't']; // file contains: a, b
/// }
/// assert_eq!(my_module::T, (0, 10));
/// ```
#[proc_macro]
pub fn inject(input: TokenStream) -> TokenStream {
    let mut tt: TokenStream2 = input.into();

    match replace_tree_in_stream(&mut tt, |iter| {
        autoinject_tt_in_group(&mut 0, iter, &mut None)
    }) {
        SearchGroup::Error(e) => e.into(),
        SearchGroup::Break => tt.into(),
    }
}
