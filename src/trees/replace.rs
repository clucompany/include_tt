use alloc::vec::Vec;
use core::slice::IterMut;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree as TokenTree2};

/*
    TODO, At the moment this is not optimally done :(.
*/

/// Helper for in-place replacement of tokens within a Group.
/// 
/// Note: Due to `proc_macro2` API limitations, this performs a full 
/// round-trip: Stream -> Vec -> Stream.
pub fn replace_tree_in_group<R>(
    real_group: &mut Group,
    next: impl FnOnce(IterMut<'_, TokenTree2>) -> R,
) -> R {
    let mut tokens: Vec<TokenTree2> = real_group.stream().into_iter().collect();
    let result = next(tokens.iter_mut());

    // Reconstruct the group with the same delimiter and span
    let mut new_group = Group::new(real_group.delimiter(), TokenStream2::from_iter(tokens));
    new_group.set_span(real_group.span());
    *real_group = new_group;

    result
}

/// Helper for in-place replacement of tokens within a TokenStream.
pub fn replace_tree_in_stream<R>(
    stream: &mut TokenStream2,
    next: impl FnOnce(IterMut<'_, TokenTree2>) -> R,
) -> R {
    // mem::take is efficient here as it leaves an empty TokenStream
    let mut tokens: Vec<TokenTree2> = std::mem::take(stream).into_iter().collect();
    let result = next(tokens.iter_mut());

    *stream = TokenStream2::from_iter(tokens);
    result
}
