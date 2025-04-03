use proc_macro2::{Delimiter, Group, TokenStream as TokenStream2, TokenTree};

/// Create an empty tree that does not affect the target tree.
///
/// Typically used to avoid deleting tree elements and replacing them with voids.
#[inline]
pub fn make_null_ttree() -> TokenTree {
	TokenTree::Group(Group::new(Delimiter::None, TokenStream2::new()))
}
