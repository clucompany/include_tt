
use proc_macro2::{TokenStream as TokenStream2, Span};
use std::io::Error as IOError;
use syn::Error as SynError;

#[derive(Debug)]
pub enum LoadFileAndAutoMakeTreeErr<'a> {
	/// The error type for I/O operations of the 
	/// [Read], [Write], [Seek], and associated traits.
	ReadToString {
		err: IOError, 
		path: &'a str,
	},
	
	/// Error returned when a Syn parser cannot parse the input tokens.
	ParseStr(SynError),
}

impl<'a> LoadFileAndAutoMakeTreeErr<'a> {
	#[inline(always)]
	pub const fn read_to_string(err: IOError, path: &'a str) -> Self {
		Self::ReadToString {
			err,
			path
		}
	}
	
	/// Convert an error to a syntax tree.
	#[inline]
	pub fn into_tt_err(self, span: Span) -> TokenStream2 {
		match self {
			Self::ReadToString { err, path } => {
				let se = format!("{:?}", err);
				sg_err! {
					[span]: "Error loading file, err: ", #se, ", path: ", #path, "."
				}
			},
			Self::ParseStr(e) => {
				let se = format!("{:?}", e);
				sg_err! {
					[span]: "Failed to convert to tree `tt`: ", #se, "."
				}
			}
		}
	}
}

#[allow(dead_code)]
/// Load the file and present it as a compiler tree set.
pub fn load_file_and_automake_tree<'a, R>(
	path: &'a str,
	
	// Preprocessing a file loaded into a String before passing it directly to the parser.
	//
	// (If this is not required, it is enough to leave the closure empty.)
	prepare_file_str: impl FnOnce(&mut String),
) -> Result<Option<TokenStream2>, LoadFileAndAutoMakeTreeErr<'a>> {
	load_file_and_automake_tree_fn(
		path,
		prepare_file_str,
		|a| Ok(a),
		|e| Err(e),
	)
}

/// Load the file and present it as a compiler tree set.
pub fn load_file_and_automake_tree_fn<'a, R>(
	path: &'a str,
	
	// Preprocessing a file loaded into a String before passing it directly to the parser.
	//
	// (If this is not required, it is enough to leave the closure empty.)
	prepare_file_str: impl FnOnce(&mut String),
	
	next: impl FnOnce(Option<TokenStream2>) -> R,
	err: impl FnOnce(LoadFileAndAutoMakeTreeErr<'a>) -> R,
) -> R {
	let mut data = match std::fs::read_to_string(path) {
		Ok(a) => a,
		Err(e) => return err(LoadFileAndAutoMakeTreeErr::ReadToString {
			err: e, 
			path: path,
		}),
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
