
use core::slice::IterMut;
use proc_macro2::{Group, TokenTree as TokenTree2, TokenStream as TokenStream2};
use alloc::vec::Vec;

/*
	TODO, At the moment this is not optimally done :(.
*/

/// A small function that provides the ability 
/// to iterate and replace trees in place.
pub fn support_replace_tree_in_group<R>(
	real_group: &mut Group,
	
	next: impl FnOnce(IterMut<TokenTree2>) -> R
) -> R {
	let span = real_group.span();
	let delimeter = real_group.delimiter();
	
	let mut allts: Vec<TokenTree2> = 
		real_group.stream()
		.into_iter()
		.collect();
	
	let result = next(
		allts.iter_mut()
	);
	
	let mut ngroup = Group::new(
		delimeter,
		TokenStream2::from_iter(allts)
	);
	ngroup.set_span(span);
	*real_group = ngroup;
	
	result
}

/// A small function that provides the ability 
/// to iterate and replace trees in place.
/// 
/// For a group, use the `support_replace_tree_in_group` function.
pub fn support_replace_tree_in_stream<R>(
	stream: &mut TokenStream2,
	
	next: impl FnOnce(IterMut<TokenTree2>) -> R
) -> R {
	let mut allts: Vec<TokenTree2> = 
		core::mem::take(stream)
		.into_iter()
		.collect();
	
	let result = next(
		allts.iter_mut()
	);
	
	*stream = TokenStream2::from_iter(allts);
	result
}
