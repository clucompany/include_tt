
use std::slice::IterMut;
use proc_macro2::{Group, Delimiter, TokenTree as TokenTree2};
use crate::{TreeResult, trees::sg_err};
use std::fmt::Write;

/// This function allows you to correctly end a group with 
/// a delimiter () and skip ';' if it is needed.
#[allow(dead_code)]
pub (crate) fn check_correct_endgroup<'i>(
	group: &'_ Group,
	
	iter: &mut IterMut<'i, TokenTree2>,
	
	endgroup: &[char],
) -> TreeResult<Option<&'i mut TokenTree2>> {
	let group_span = group.span();
	
	/// Assembly &[char] array into final string `A`, `B`, `C`
	fn make_endroup_str(endgroup: &[char]) -> String {
		let mut str = String::with_capacity(endgroup.len() * 3);
		
		let mut iter = endgroup.iter();
		if let Some(a) = iter.next() {
			if let Err(..) = write!(str, "`{}`", a) {
				return str;
			}
			while let Some(a) = iter.next() {
				if let Err(..) = write!(str, ", `{}`", a) {
					return str;
				}
			}
		}
		
		str
	}
	
	match group.delimiter() {
		Delimiter::Parenthesis => {
			let optm_punct = iter.next();
			
			if let Some(ref m_punct) = optm_punct {
				match m_punct {
					TokenTree2::Punct(ref punct) => {
						let is_valid = 'is_valid: {
							let a_punct = punct.as_char();
							for a_endgroup in endgroup {
								if &a_punct == a_endgroup {
									break 'is_valid true;
								}
							}
							
							break 'is_valid false;
						};
						if !is_valid {
							let e_group_str = make_endroup_str(endgroup);
							sg_err! {
								return [punct.span()]: "", #e_group_str, " was expected."
							}
						}
						
						return TreeResult::Ok(optm_punct);
					},
					
					_ => {
						let e_group_str = make_endroup_str(endgroup);
						sg_err! {
							return [m_punct.span()]: "", #e_group_str, " was expected."
						}
					},
				}
			} else {
				let e_group_str = make_endroup_str(endgroup);
				sg_err! {
					return [group_span]: "", #e_group_str, " was expected."
				}
			}
		},
		Delimiter::Brace => return TreeResult::Ok(None), // ok
		Delimiter::Bracket | Delimiter::None => {
			sg_err! {
				return [group.span()]: "Unsupported group type."
			}
		},
	}
}
