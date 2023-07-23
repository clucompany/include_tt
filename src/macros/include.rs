
use std::{io::Read, fs::File};
use proc_macro2::{TokenTree as TokenTree2, Group, TokenStream as TokenStream2, Delimiter, Span, Literal};
use crate::{trees::{sg_err, result::TreeResult, loader::{load_file_and_automake_tree, LoadFileAndAutoMakeTreeErr}}, exprs::literal::ExprLit};

/// A trait that specifies the final behavior for the `include` macro.
pub trait BehMacroInclude {
	type Result;
	
	/// Assembly of the final tree.
	fn make_tree(
		arg0: &ExprLit,
		
		group_span: Span,
		literal_span: Span
	) -> TreeResult<Option<Self::Result>>;
}

/// Easily include trees from a file in your 
/// final custom macro code.
pub (crate) enum IncludeTt {}

impl BehMacroInclude for IncludeTt {
	type Result = TokenTree2;

	fn make_tree(
		sspath: &ExprLit,
		
		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Option<Self::Result>> {
		load_file_and_automake_tree(
			&sspath,
			
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
		) 
	}
}

/// Includes the entire file as a single line, 
/// similar to 'include_str'.
pub (crate) enum IncludeStr {}

impl BehMacroInclude for IncludeStr {
	type Result = TokenTree2;

	fn make_tree(
		sspath: &ExprLit,
		
		group_span: Span, 
		literal_span: Span
	) -> TreeResult<Option<Self::Result>> {
		let data = match std::fs::read_to_string(sspath.as_str()) {
			Ok(a) => a,
			Err(e) => return LoadFileAndAutoMakeTreeErr::ReadToString(e)
				.into_tt_err(literal_span)
				.into(),
		};
		
		let mut lit = Literal::string(&data);
		lit.set_span(group_span);
		
		return TreeResult::Ok(
			Some(TokenTree2::Literal(lit))
		);
	}
}

/// Includes the entire file as a binary array, 
/// similar to 'include_str'.
pub (crate) enum IncludeArr {}

impl BehMacroInclude for IncludeArr {
	type Result = TokenTree2;

	fn make_tree(
		sspath: &ExprLit,
		
		group_span: Span, 
		literal_span: Span
	) -> TreeResult<Option<Self::Result>> {
		let vec = {
			let mut file = match File::open(sspath.as_str()) {
				Ok(a) => a,
				Err(e) => return LoadFileAndAutoMakeTreeErr::ReadToString(e)
					.into_tt_err(literal_span)
					.into(),
			};
			
			let mut vec = Vec::new(); // capacity is not required.
			if let Err(e) = file.read_to_end(&mut vec) {
				return LoadFileAndAutoMakeTreeErr::ReadToString(e)
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
	}
}

/// Build macro `include`/`include_str`/`include_arr`.
pub fn macro_rule_include<'mflush, A>(
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
		if let TokenTree2::Literal(literal) = g_stream {
			let fstream = {
				let sspath = literal.to_string();
				let lspan = literal.span();
				
				ExprLit::try_new_fn(
					&sspath,
					|sspath| {
						A::make_tree(
							sspath, 
							group.span(),
							lspan
						)
					},
					|e| TreeResult::Err(e.into_tt_err(lspan)),
				)
			};
			
			return fstream;
		}else {
			sg_err! {
				return [g_stream.span()]: "The expected path must be written in string format.."
			}
		}
	}
	
	TreeResult::Ok(None)
}