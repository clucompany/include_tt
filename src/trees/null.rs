use proc_macro2::{Delimiter, Group, Span, TokenStream as TokenStream2, TokenTree as TokenTree2};

/// Create an empty tree that does not affect the target tree.
///
/// Typically used to avoid deleting tree elements and replacing them with voids.
#[allow(dead_code)]
pub fn make_null_ttree() -> TokenTree2 {
	let ngroup = Group::new(Delimiter::None, TokenStream2::new());

	TokenTree2::Group(ngroup)
}

/// Creates an empty group and binds it to the span
pub fn make_null_group(group_span: Span) -> TokenTree2 {
	let mut ngroup = Group::new(Delimiter::None, TokenStream2::new());
	ngroup.set_span(group_span);

	TokenTree2::Group(ngroup)
}
