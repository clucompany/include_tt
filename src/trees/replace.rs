
use std::slice::IterMut;
use proc_macro2::{Group, TokenTree as TokenTree2, TokenStream as TokenStream2};

/// A small function that provides the ability 
/// to iterate and replace trees in place.
pub fn support_replace_tree_in_group<R>(
	real_group: &mut Group,
	
	next: impl FnOnce(IterMut<TokenTree2>) -> R
) -> R {
	let span = real_group.span();
	let delimeter = real_group.delimiter();
	
	let mut allts: Vec<TokenTree2> = 
		std::mem::replace(
			&mut real_group.stream(),
			Default::default()
		)
		.into_iter()
		.collect();
	
	let result = next(allts.iter_mut());
	
	let mut ngroup = Group::new(
		delimeter,
		TokenStream2::from_iter(allts.into_iter())
	);
	ngroup.set_span(span);
	*real_group = ngroup;
	
	result
}

/// A small function that provides the ability 
/// to iterate and replace trees in place.
/// 
/// For a group, use the `replace_tree_in_group` function.
pub fn support_replace_tree_in_stream<R>(
	stream: &mut TokenStream2,
	
	next: impl FnOnce(IterMut<TokenTree2>) -> R
) -> R {
	let mut allts: Vec<TokenTree2> = 
		std::mem::replace(stream, Default::default())
		.into_iter()
		.collect();
	
	let result = next(allts.iter_mut());
	
	*stream = TokenStream2::from_iter(allts.into_iter());
	result
}
