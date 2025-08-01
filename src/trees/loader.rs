use alloc::{format, string::String};
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::{borrow::Cow, io::Error as IOError, path::Path};
use syn::Error as SynError;

/// Variants of errors when loading a file and presenting it as a set of compiler trees.
#[derive(Debug)]
pub enum LoadFileAndAutoMakeTreeErr<'a> {
	/// The error type for I/O operations of the
	/// [Read], [Write], [Seek], and associated traits.
	ReadToString { err: IOError, path: Cow<'a, Path> },

	/// Error returned when a Syn parser cannot parse the input tokens.
	ParseStr(SynError),
}

impl<'a> LoadFileAndAutoMakeTreeErr<'a> {
	/// The error type for I/O operations of the
	/// [Read], [Write], [Seek], and associated traits.
	#[inline]
	pub const fn read_to_string(err: IOError, path: Cow<'a, Path>) -> Self {
		Self::ReadToString { err, path }
	}

	/// Convert an error to a syntax tree.
	pub fn into_tt_err(self, span: Span) -> TokenStream2 {
		match self {
			Self::ReadToString { err, path } => {
				let spath = format!("{path:?}"); // TODO REFACTORME
				let se = format!("{err:?}");
				sg_err! {
					[span]: "Error loading file, err: '", #se, "', path: ", #spath, "."
				}
			}
			Self::ParseStr(e) => {
				let se = format!("{e:?}");
				sg_err! {
					[span]: "Failed to convert to tree `tt`: '", #se, "'."
				}
			}
		}
	}
}

#[allow(dead_code)]
/// Load the file and present it as a compiler tree set.
pub fn load_file_and_automake_tree(
	path: &Path,

	// Preprocessing a file loaded into a String before passing it directly to the parser.
	//
	// (If this is not required, it is enough to leave the closure empty.)
	prepare_file_str: impl FnOnce(&mut String),
) -> Result<Option<TokenStream2>, LoadFileAndAutoMakeTreeErr> {
	load_file_and_automake_tree_with_fns(path, prepare_file_str, Ok, Err)
}

/// Load the file and present it as a compiler tree set.
pub fn load_file_and_automake_tree_with_fns<'a, R>(
	path: &'a Path,

	// Preprocessing a file loaded into a String before passing it directly to the parser.
	//
	// (If this is not required, it is enough to leave the closure empty.)
	prepare_file_str: impl FnOnce(&mut String),

	next: impl FnOnce(Option<TokenStream2>) -> R,
	err: impl FnOnce(LoadFileAndAutoMakeTreeErr<'a>) -> R,
) -> R {
	let mut data = match std::fs::read_to_string(path) {
		Ok(a) => a,
		Err(e) => {
			let path = path
				.canonicalize()
				.map_or_else(|_| Cow::Borrowed(path), Cow::Owned);
			return err(LoadFileAndAutoMakeTreeErr::read_to_string(e, path));
		}
	};

	if data.is_empty() {
		return next(None);
	}

	prepare_file_str(&mut data);

	match syn::parse_str(&data) {
		Ok(a) => next(Some(a)),
		Err(e) => err(LoadFileAndAutoMakeTreeErr::ParseStr(e)),
	}
}
