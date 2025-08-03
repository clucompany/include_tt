use proc_macro2::TokenStream as TokenStream2;

/// Events for the `autoinject_tt_in_group` function.
pub(crate) enum SearchGroup {
	/// Abort the group search
	/// and display an error.
	Error(TokenStream2),

	/// The trees are over,
	/// we need to stop the search.
	Break,
}

impl From<TokenStream2> for SearchGroup {
	#[inline]
	fn from(e: TokenStream2) -> Self {
		Self::Error(e)
	}
}
