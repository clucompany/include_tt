
use proc_macro2::{TokenStream as TokenStream2, Span};
use std::io::Error as IOError;
use syn::Error as SynError;

#[derive(Debug)]
pub (crate) enum LoadFileAndAutoMakeTreeErr {
	ReadToString(IOError),
	ParseStr(SynError),
}

impl LoadFileAndAutoMakeTreeErr {
	/// Convert an error to a syntax tree.
	#[inline]
	pub fn into_tt_err(self, span: Span) -> TokenStream2 {
		match self {
			Self::ReadToString(e) => {
				let se = format!("{:?}", e);
				sg_err! {
					[span]: "Error loading file, err: ", #se, "."
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

/// Load the file and present it as a compiler tree set.
pub (crate) fn load_file_and_automake_tree<R>(
	path: &str,
	
	next: impl FnOnce(TokenStream2) -> R,
	err: impl FnOnce(LoadFileAndAutoMakeTreeErr) -> R,
) -> R {
	let data = match std::fs::read_to_string(path) {
		Ok(a) => a,
		Err(e) => return err(LoadFileAndAutoMakeTreeErr::ReadToString(e)),
	};
	
	if data.is_empty() {
		return next(Default::default());
	}
	
	match syn::parse_str(&data) {
		Ok(a) => next(a),
		Err(e) => return err(LoadFileAndAutoMakeTreeErr::ParseStr(e)),
	}
}
