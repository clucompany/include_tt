use crate::{
	PointTrack,
	exprs::literal::ExprLit,
	trees::{
		group::group_stringify_with_fns,
		loader::{LoadFileAndAutoMakeTreeErr, load_file_and_automake_tree_with_fns},
		null::make_null_group,
		result::TreeResult,
		throw_sg_err,
	},
};
use proc_macro2::{
	Delimiter, Group, Literal, Span, TokenStream as TokenStream2, TokenTree as TokenTree2,
};
use std::{borrow::Cow, io::Error as IOError, path::Path};
use std::{fs::File, io::Read};

/// A trait that specifies the final behavior for the `include` macro.
pub trait BehMacroInclude {
	/// The result of building the tree, basically `TokenTree2`.
	type Result;

	/// Assembly of the final tree.
	fn make_tree(
		arg0: &ExprLit,
		point_track_file: Option<&mut PointTrack>,
		//
		group_span: Span,
		// `span` indicating a literal occurrence or group describing a future path.
		literal_span: Span,
	) -> TreeResult<Self::Result>;

	/// Create an empty valid tree.
	fn make_empty_tree(group_span: Span) -> Self::Result;
}

/// Easily include trees from a file in your
/// final custom macro code.
pub enum IncludeTT {}

impl BehMacroInclude for IncludeTT {
	type Result = TokenTree2;

	#[inline]
	fn make_empty_tree(group_span: Span) -> Self::Result {
		make_null_group(group_span)
	}

	fn make_tree(
		sspath: &ExprLit,
		point_track: Option<&mut PointTrack>,

		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Self::Result> {
		let path = Path::new(sspath.as_str());
		load_file_and_automake_tree_with_fns(
			path,
			point_track,
			|_| {}, /* skip_prepare */
			|fs_tt| {
				let ett = fs_tt.map_or_else(TokenStream2::new, TokenStream2::from_iter);

				let mut ngroup = Group::new(Delimiter::None, ett);
				ngroup.set_span(group_span);

				TreeResult::Ok(TokenTree2::Group(ngroup))
			},
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Regular macro `include_tt` with find and replace
/// invalid tokens breaking the parser.
///
/// (Implemented specifically for C-like languages using `\` as a line code string)
pub enum IncludeTTAndFixUnkStartToken {}

impl BehMacroInclude for IncludeTTAndFixUnkStartToken {
	type Result = TokenTree2;

	#[inline]
	fn make_empty_tree(group_span: Span) -> Self::Result {
		make_null_group(group_span)
	}

	fn make_tree(
		sspath: &ExprLit,
		point_track: Option<&mut PointTrack>,

		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Self::Result> {
		let sspath = Path::new(sspath);
		load_file_and_automake_tree_with_fns(
			sspath,
			point_track,
			|p_string| {
				/* fix unk start token */
				let mut p_str = p_string.as_mut();
				while let Some(pos) = p_str.find('\\' /* one symb */) {
					let right = unsafe {
						debug_assert!(p_str.get(pos..).is_some());

						p_str.get_unchecked_mut(pos..)
					};

					if right.len() >= 2 {
						let (c_symbol, new_pstr) = right.split_at_mut(2);
						let c_array = unsafe { c_symbol.as_bytes_mut() };
						debug_assert_eq!(c_array.len(), 2);
						debug_assert_eq!(
							{
								#[allow(clippy::get_first)]
								// why?, this is done to be completely analogous to an unsafe function.
								c_array.get(0)
							},
							Some(&b'\\')
						);
						debug_assert!(c_array.get(1).is_some());

						match unsafe { c_array.get_unchecked(1) } {
							b'\n' | b'\t' | b'\r' | b' ' => {
								// This is generally safe as the '/'
								// characters were found using utf-8 lookups.
								let a_repl = unsafe { c_array.get_unchecked_mut(0) };
								*a_repl = b' ';
							}
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
					} else {
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
				let ett = fs_tt.map_or_else(TokenStream2::new, TokenStream2::from_iter);

				let mut ngroup = Group::new(Delimiter::None, ett);
				ngroup.set_span(group_span);

				TreeResult::Ok(TokenTree2::Group(ngroup))
			},
			|e| TreeResult::Err(e.into_tt_err(literal_span)),
		)
	}
}

/// Includes the entire file as a single line,
/// similar to 'include_str'.
pub enum IncludeStr {}

impl BehMacroInclude for IncludeStr {
	type Result = TokenTree2;

	fn make_empty_tree(group_span: Span) -> Self::Result {
		let mut lit = Literal::string("");
		lit.set_span(group_span);

		TokenTree2::Literal(lit)
	}

	fn make_tree(
		sspath: &ExprLit,
		point_track: Option<&mut PointTrack>,

		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Self::Result> {
		let path = Path::new(sspath);

		match std::fs::read_to_string(path) {
			Ok(data) => {
				if let Some(point_track) = point_track {
					point_track.append_track_file(path);
				}
				let mut lit = Literal::string(&data);
				lit.set_span(group_span);

				TreeResult::Ok(TokenTree2::Literal(lit))
			}
			Err(e) => {
				let path = path
					.canonicalize()
					.map_or_else(|_| Cow::Borrowed(path), Cow::Owned);

				TreeResult::Err(
					LoadFileAndAutoMakeTreeErr::read_to_string(e, path).into_tt_err(literal_span),
				)
			}
		}
	}
}

/// Includes the entire file as a binary array,
/// similar to 'include_str'.
pub enum IncludeArr {}

impl BehMacroInclude for IncludeArr {
	type Result = TokenTree2;

	fn make_empty_tree(group_span: Span) -> Self::Result {
		let mut lit = Literal::byte_string(&[]);
		lit.set_span(group_span);

		TokenTree2::Literal(lit)
	}

	fn make_tree(
		sspath: &ExprLit,
		point_track: Option<&mut PointTrack>,

		group_span: Span,
		literal_span: Span,
	) -> TreeResult<Self::Result> {
		let path = Path::new(sspath);
		let vec = {
			let make_err = |e: IOError| {
				let path = path
					.canonicalize()
					.map_or_else(|_| Cow::Borrowed(path), Cow::Owned);
				TreeResult::from(
					LoadFileAndAutoMakeTreeErr::read_to_string(e, path).into_tt_err(literal_span),
				)
			};
			let mut file = match File::open(path) {
				Ok(a) => a,
				Err(e) => return make_err(e),
			};

			let mut vec = Vec::new(); // capacity is not required.
			if let Err(e) = file.read_to_end(&mut vec) {
				return make_err(e);
			};

			vec
		};

		if let Some(point_track) = point_track {
			point_track.append_track_file(path);
		}
		let mut lit = Literal::byte_string(&vec);
		lit.set_span(group_span);

		TreeResult::Ok(TokenTree2::Literal(lit))
	}
}

/// Build macro `include`/`include_str`/`include_arr`.
pub fn macro_rule_include<A>(
	group: &'_ Group,
	point_track: Option<&mut PointTrack>,
) -> TreeResult<A::Result>
where
	A: BehMacroInclude,
{
	let stream0 = {
		let all_streams = group.stream();
		let mut iter = all_streams.into_iter();

		let stream0 = iter.next();
		if let Some(unk) = iter.next() {
			throw_sg_err! {
				return [unk.span()]: "Specify a valid path to the file written with `\"/Test.tt\"`, or `'T'`, or use a group of different trees `[/, \"Test\", '/']`."
			}
		}

		stream0
	};

	match stream0 {
		None => TreeResult::Ok(A::make_empty_tree(group.span())),
		Some(TokenTree2::Group(g_stream)) => {
			// The path is a group of TokenTrees that can be converted to
			// a string and concatenated.

			group_stringify_with_fns(
				&g_stream,
				|stringify| {
					let exprlit = unsafe { ExprLit::new_unchecked(&stringify) };

					A::make_tree(
						// The value is already ready to be used as a path.
						exprlit,
						point_track,
						group.span(),
						g_stream.span(),
					)
				},
				// Empty
				|| TreeResult::Ok(A::make_empty_tree(group.span())),
				// Err
				TreeResult::Err,
			)
		}
		Some(TokenTree2::Literal(literal)) => {
			// The path is a single string

			ExprLit::try_new_with_fns(
				&literal.to_string(),
				|slit| {
					A::make_tree(
						// Can be `"Test"` or `'T'` (with actual quotes in the value)
						// and may require character escaping to be handled.
						slit,
						point_track,
						group.span(),
						literal.span(),
					)
				},
				|e| TreeResult::Err(e.into_tt_err(literal.span())),
			)
		}
		Some(g_stream) => throw_sg_err! {
			return [g_stream.span()]: "The path was expected as a single string (example: \"../test.tt\") or a path formatted as separate TokenTrees (example: ['.' '.' test \".tt\"])."
		},
	}
}
