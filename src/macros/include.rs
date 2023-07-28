
use std::{io::Read, fs::File};
use proc_macro2::{TokenTree as TokenTree2, Group, TokenStream as TokenStream2, Delimiter, Span, Literal};
use crate::{trees::{sg_err, result::TreeResult, loader::{load_file_and_automake_tree, LoadFileAndAutoMakeTreeErr}, ttry, group::g_stringify}, exprs::literal::{ExprLit, ExprLitTryNewErr}};

/// A trait that specifies the final behavior for the `include` macro.
pub trait BehMacroInclude {
	type Result;
	
	/// Assembly of the final tree.
	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span,
		literal_span: Span
	) -> TreeResult<Option<Self::Result>>;
}

#[derive(Debug)]
pub enum BehMacroArg0 {
	/// Raw literal value
	ExpMakeExprLit(String),
	
	/// The value obtained by merging groups with different trees.
	Stringify(String),
}

impl BehMacroArg0 {
	#[inline]
	pub fn get_str<R>(
		&self, 
		
		next: impl FnOnce(&'_ str) -> R, 
		err: impl FnOnce(ExprLitTryNewErr) -> R
	) -> R {
		match self {
			Self::Stringify(a) => next(&a),
			Self::ExpMakeExprLit(a) => {
				ExprLit::try_new_search_and_autoreplaceshielding_fn(
					a, 
					|a| next(&a),
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

	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Option<Self::Result>> {
		arg0.get_str(
			|sspath| load_file_and_automake_tree(
				sspath,
				
				|_| {},
				|fs_tt| {
					let mut ngroup = Group::new(
						Delimiter::None,
						TokenStream2::from_iter(fs_tt.into_iter())
					);
					ngroup.set_span(group_span);
					
					return TreeResult::Ok(
						Some(TokenTree2::Group(ngroup))
					);
				},
				|e| TreeResult::Err(e.into_tt_err(literal_span)),
			) , 
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// The usual macro `include_tt` with the search and replacement 
/// of invalid tokens, breaking the parser.
/// 
/// 1. Replacing single characters '\' with spaces. Note that if '\\' is used, 
/// no substitution occurs. (This character aborts the parser with an "unknown start token" 
/// error.)
pub (crate) enum IncludeTtAndFixUnkStartToken {}

impl BehMacroInclude for IncludeTtAndFixUnkStartToken {
	type Result = TokenTree2;

	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Option<Self::Result>> {
		arg0.get_str(
			|sspath| load_file_and_automake_tree(
				sspath,
				
				|p_string| { /*fix unk start token*/
					let mut p_str = p_string.as_mut();
					while let Some(pos) = p_str.find('\\') {
						let (_, right) = p_str.split_at_mut(pos);
						
						if right.len() > 2 {
							let (c_symbol, new_pstr) = right.split_at_mut(2);
							if c_symbol != "\\" {
								let array = unsafe { c_symbol.as_bytes_mut() }.iter_mut();
								for a in array {
									*a = b' ';
								}
							}
							
							p_str = new_pstr;
							if p_str.is_empty() {
								break;
							}
						}else {
							let array = unsafe { right.as_bytes_mut() }.iter_mut();
							for a in array {
								*a = b' ';
							}
							break;
						}
					}
				},
				|fs_tt| {
					let mut ngroup = Group::new(
						Delimiter::None,
						TokenStream2::from_iter(fs_tt.into_iter())
					);
					ngroup.set_span(group_span);
					
					return TreeResult::Ok(
						Some(TokenTree2::Group(ngroup))
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

	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span, 
		literal_span: Span
	) -> TreeResult<Option<Self::Result>> {
		arg0.get_str(
			|sspath| {
				let data = match std::fs::read_to_string(sspath) {
					Ok(a) => a,
					Err(e) => return LoadFileAndAutoMakeTreeErr::ReadToString {
						err: e,
						path: sspath
					}
					.into_tt_err(literal_span)
					.into(),
				};
				
				let mut lit = Literal::string(&data);
				lit.set_span(group_span);
				
				return TreeResult::Ok(
					Some(TokenTree2::Literal(lit))
				);
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

	fn make_tree(
		arg0: BehMacroArg0,
		
		group_span: Span, 
		literal_span: Span
	) -> TreeResult<Option<Self::Result>> {
		arg0.get_str(
			|sspath| {
				let vec = {
					let mut file = match File::open(sspath) {
						Ok(a) => a,
						Err(e) => return LoadFileAndAutoMakeTreeErr::ReadToString {
							err: e,
							path: sspath
						}
						.into_tt_err(literal_span)
						.into(),
					};
					
					let mut vec = Vec::new(); // capacity is not required.
					if let Err(e) = file.read_to_end(&mut vec) {
						return LoadFileAndAutoMakeTreeErr::ReadToString {
							err: e,
							path: sspath
						}
						.into_tt_err(literal_span)
						.into();
					};
					
					vec
				};
				
				let mut lit = Literal::byte_string(&vec);
				lit.set_span(group_span);
				
				return TreeResult::Ok(
					Some(TokenTree2::Literal(lit))
				);
			},
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Build macro `include`/`include_str`/`include_arr`.
pub fn macro_rule_include<A>(
	group: &'_ Group,
) -> TreeResult<Option<A::Result>> where A: BehMacroInclude {
	let mut stream = group.stream().into_iter();
	
	let stream0 = stream.next();
	if let Some(unk) = stream.next() {
		sg_err! {
			return [unk.span()]: "Please specify the correct path to the file."
		}
	}
	
	if let Some(g_stream) = stream0 {
		// The path is a group of TokenTrees that can be converted to 
		// a string and concatenated.
		if let TokenTree2::Group(group) = g_stream {
			if let Some(stringify) = ttry!(g_stringify(&group)) {
				let lspan = group.span();
				
				let fstream = A::make_tree(
					BehMacroArg0::Stringify(stringify),
					group.span(),
					lspan
				);
				
				return fstream;
			}
		} else
		// The path is a single string
		if let TokenTree2::Literal(literal) = g_stream {
			let lspan = literal.span();
			
			let fstream = A::make_tree(
				BehMacroArg0::ExpMakeExprLit(literal.to_string()),
				group.span(),
				lspan
			);
			
			return fstream;
		}else {
			sg_err! {
				return [g_stream.span()]: "The path was expected as a single string (example: \"../test.tt\") or a path formatted as separate TokenTrees (example: ['.' '.' test \".tt\"])."
			}
		}
	}
	
	TreeResult::Ok(None)
}