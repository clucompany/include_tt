
use std::{io::Read, fs::File};
use proc_macro2::{TokenTree as TokenTree2, Group, TokenStream as TokenStream2, Delimiter, Span, Literal};
use crate::{trees::{sg_err, result::TreeResult, loader::{load_file_and_automake_tree_fn, LoadFileAndAutoMakeTreeErr}, ttry, group::g_stringify}, exprs::literal::{ExprLit, ExprLitTryNewErr}};

/// A trait that specifies the final behavior for the `include` macro.
pub trait BehMacroInclude {
	/// The result of building the tree, basically `TokenTree2`.
	type Result;
	
	/// Assembly of the final tree.
	fn make_tree(
		arg0: BehMacroArg0,
		
		// 
		group_span: Span,
		// `span` indicating a literal occurrence or group describing a future path.
		literal_span: Span
	) -> TreeResult<Self::Result>;
	
	/// Create an empty valid tree.
	fn make_empty_tree(
		group_span: Span,
	) -> Self::Result;
}

/// An argument, basically a path, written in `literals` or created with 
/// a `group` via `stringify`.
#[derive(Debug)]
pub enum BehMacroArg0 {
	/// Raw literal value.
	ExpMakeExprLit(String),
	
	/// The value obtained by merging groups with different trees.
	Stringify(String),
}

impl BehMacroArg0 {
	/// Get a data string without special characters, 
	/// with final escaping (if required).
	#[inline]
	pub fn get_str_fn<R>(
		&self,
		
		next: impl FnOnce(&'_ str) -> R, 
		err: impl FnOnce(ExprLitTryNewErr) -> R
	) -> R {
		match self {
			Self::Stringify(a) => next(&a),
			Self::ExpMakeExprLit(a) => {
				ExprLit::try_new_fn(
					a, 
					|a| {
						let result = next(&a);
						drop(a);
						
						result
					},
					err
				)
			}
		}
	}
}

/// Easily include trees from a file in your 
/// final custom macro code.
pub (crate) enum IncludeTt {}

impl BehMacroInclude for IncludeTt {
	type Result = TokenTree2;
	
	fn make_empty_tree(group_span: Span) -> Self::Result {
		let mut ngroup = Group::new(
			Delimiter::None,
			TokenStream2::new(),
		);
		ngroup.set_span(group_span);
		
		TokenTree2::Group(ngroup)
	}
	
	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Self::Result> {
		arg0.get_str_fn(
			|sspath| load_file_and_automake_tree_fn(
				sspath,
				
				|_| {},
				|fs_tt| {
					let ett = match fs_tt {
						Some(a) => TokenStream2::from_iter(a.into_iter()),
						None => TokenStream2::new(),
					};
					
					let mut ngroup = Group::new(
						Delimiter::None,
						ett,
					);
					ngroup.set_span(group_span);
					
					return TreeResult::Ok(
						TokenTree2::Group(ngroup)
					);
				},
				|e| TreeResult::Err(e.into_tt_err(literal_span)),
			) , 
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Regular macro `include_tt` with find and replace
/// invalid tokens breaking the parser.
///
/// (Implemented specifically for C-like languages using `\` as a line code string)
pub (crate) enum IncludeTtAndFixUnkStartToken {}

impl BehMacroInclude for IncludeTtAndFixUnkStartToken {
	type Result = TokenTree2;
	
	fn make_empty_tree(group_span: Span) -> Self::Result {
		let mut ngroup = Group::new(
			Delimiter::None,
			TokenStream2::new(),
		);
		ngroup.set_span(group_span);
		
		TokenTree2::Group(ngroup)
	}
	
	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Self::Result> {
		arg0.get_str_fn(
			|sspath| load_file_and_automake_tree_fn(
				sspath,
				
				|p_string| { /*fix unk start token*/
					let mut p_str = p_string.as_mut();
					while let Some(pos) = p_str.find('\\' /* one symb */) {
						let right = unsafe {
							debug_assert_eq!(p_str.get(pos..).is_some(), true);
							
							p_str.get_unchecked_mut(pos..)
						};
						
						if right.len() >= 2 {
							let (c_symbol, new_pstr) = right.split_at_mut(2);
							let c_array = unsafe {
								c_symbol.as_bytes_mut()
							};
							debug_assert_eq!(c_array.len(), 2);
							debug_assert_eq!(c_array.get(0), Some(&b'\\'));
							debug_assert_eq!(c_array.get(1).is_some(), true);
							
							match unsafe { c_array.get_unchecked(1) } {
								b'\n' | b'\t' | b'\r' | b' ' => {
									// This is generally safe as the '/' 
									// characters were found using utf-8 lookups.
									let a_repl = unsafe { c_array.get_unchecked_mut(0) };
									*a_repl = b' ';
								},
								/*b'\\' => {
									// This is generally safe as the '/' 
									// characters were found using utf-8 lookups.
									for a in c_array {
										*a = b' ';
									}
								},*/
								_ => {}
							}
							
							if new_pstr.is_empty() {
								break;
							}
							p_str = new_pstr;
						}else {
							// This is generally safe as the '/'
							// characters were found using utf-8 lookups.
							let array = unsafe { right.as_bytes_mut() }.iter_mut();
							for a in array {
								*a = b' ';
							}
							
							break;
						}
					}
				},
				|fs_tt| {
					let ett = match fs_tt {
						Some(a) => TokenStream2::from_iter(a.into_iter()),
						None => TokenStream2::new(),
					};
					
					let mut ngroup = Group::new(
						Delimiter::None,
						ett
					);
					ngroup.set_span(group_span);
					
					return TreeResult::Ok(
						TokenTree2::Group(ngroup)
					);
				},
				|e| TreeResult::Err(e.into_tt_err(literal_span)),
			) , 
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Includes the entire file as a single line, 
/// similar to 'include_str'.
pub (crate) enum IncludeStr {}

impl BehMacroInclude for IncludeStr {
	type Result = TokenTree2;

	fn make_empty_tree(group_span: Span) -> Self::Result {
		let mut lit = Literal::string("");
		lit.set_span(group_span);
		
		TokenTree2::Literal(lit)
	}
	
	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span, 
		literal_span: Span
	) -> TreeResult<Self::Result> {
		arg0.get_str_fn(
			|sspath| {
				let data = match std::fs::read_to_string(sspath) {
					Ok(a) => a,
					Err(e) => return LoadFileAndAutoMakeTreeErr::read_to_string(e, sspath)
						.into_tt_err(literal_span)
						.into(),
				};
				
				let mut lit = Literal::string(&data);
				lit.set_span(group_span);
				
				TreeResult::Ok(
					TokenTree2::Literal(lit)
				)
			},
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Includes the entire file as a binary array, 
/// similar to 'include_str'.
pub (crate) enum IncludeArr {}

impl BehMacroInclude for IncludeArr {
	type Result = TokenTree2;

	fn make_empty_tree(group_span: Span) -> Self::Result {
		let mut lit = Literal::byte_string(&[]);
		lit.set_span(group_span);
		
		TokenTree2::Literal(lit)
	}
	
	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span, 
		literal_span: Span
	) -> TreeResult<Self::Result> {
		arg0.get_str_fn(
			|sspath| {
				let vec = {
					let mut file = match File::open(sspath) {
						Ok(a) => a,
						Err(e) => return LoadFileAndAutoMakeTreeErr::read_to_string(e, sspath)
							.into_tt_err(literal_span)
							.into(),
					};
					
					let mut vec = Vec::new(); // capacity is not required.
					if let Err(e) = file.read_to_end(&mut vec) {
						return LoadFileAndAutoMakeTreeErr::read_to_string(e, sspath)
							.into_tt_err(literal_span)
							.into();
					};
					
					vec
				};
				
				let mut lit = Literal::byte_string(&vec);
				lit.set_span(group_span);
				
				return TreeResult::Ok(
					TokenTree2::Literal(lit)
				);
			},
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Build macro `include`/`include_str`/`include_arr`.
pub fn macro_rule_include<A>(
	group: &'_ Group,
) -> TreeResult<A::Result> where A: BehMacroInclude {
	let stream0 = {
		let all_streams = group.stream();
		let mut iter = all_streams.into_iter();
	
		let stream0 = iter.next();
		if let Some(unk) = iter.next() {
			sg_err! {
				return [unk.span()]: "Specify a valid path to the file written with `\"/Test.tt\"`, or `'T'`, or use a group of different trees `[/, \"Test\", '/']`."
			}
		}
		
		stream0
	};
	
	match stream0 {
		None => TreeResult::Ok( A::make_empty_tree(group.span()) ),
		Some(TokenTree2::Group(g_stream)) => {
			// The path is a group of TokenTrees that can be converted to 
			// a string and concatenated.
			
			match ttry!( g_stringify(&g_stream) ) {
				Some(stringify) =>  A::make_tree(
					// The value is already ready to be used as a path.
					BehMacroArg0::Stringify(stringify),
					
					group.span(),
					g_stream.span()
				),
				None => TreeResult::Ok( A::make_empty_tree(group.span()) ),
			}
		},
		Some(TokenTree2::Literal(literal)) => { 
			// The path is a single string
			
			A::make_tree(
				// Can be `"Test"` or `'T'` (with actual quotes in the value) 
				// and may require character escaping to be handled.
				BehMacroArg0::ExpMakeExprLit(literal.to_string()),
				
				group.span(),
				literal.span()
			)
		},
		Some(g_stream) => sg_err! {
			return [g_stream.span()]: "The path was expected as a single string (example: \"../test.tt\") or a path formatted as separate TokenTrees (example: ['.' '.' test \".tt\"])."
		},
	}
}