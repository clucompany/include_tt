
use proc_macro2::TokenStream as TokenStream2;

/// The most common `Result`, 
/// but with a default function to select the error, not the value.
#[derive(Debug)]
pub enum TreeResult<Ok> {
	/// An error has occurred and the entire parser must be aborted.
	Err(TokenStream2),
	/// The result of a successful operation.
	Ok(Ok),
}

impl<T> From<TokenStream2> for TreeResult<T> {
	#[inline(always)]
	fn from(e: TokenStream2) -> Self {
		Self::Err(e)
	}
}

/// The functionality repeats the `?` operator, but is written for `TreeResult`.
// TODO! Requires `try` stabilization from std.
macro_rules! ttry {
	[ $e: expr ] => {
		match $e {
			TreeResult::Ok(a) => a,
			TreeResult::Err(e) => return e.into(),
		}
	};
}
