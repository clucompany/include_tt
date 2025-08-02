use crate::{TreeResult, exprs::literal::ExprLit, throw_sg_err, trees::tq};
use alloc::{
	fmt::Write,
	format,
	string::{String, ToString},
};
use core::slice::IterMut;
use proc_macro2::{Delimiter, Group, TokenStream as TokenStream2, TokenTree as TokenTree2};

/// This function allows you to correctly end a group with
/// a delimiter () and skip ';' if it is needed.
#[allow(dead_code)]
pub fn check_correct_endgroup<'i>(
	group: &'_ Group,

	iter: &mut IterMut<'i, TokenTree2>,

	endgroup: &[char],
) -> TreeResult<Option<&'i mut TokenTree2>> {
	/// Assembly &[char] array into final string `A`, `B`, `C`
	#[inline]
	fn make_endroup_str(endgroup: &[char]) -> String {
		let mut str = String::with_capacity(endgroup.len() * 3);

		let mut iter = endgroup.iter();
		if let Some(a) = iter.next() {
			if write!(str, "`{a}`").is_err() {
				return str;
			}
			for a in iter {
				if write!(str, ", `{a}`").is_err() {
					return str;
				}
			}
		}

		str
	}

	match group.delimiter() {
		Delimiter::Parenthesis => {
			/* `( ... )` */
			let optm_punct = iter.next();

			if let Some(ref m_punct) = optm_punct {
				match m_punct {
					TokenTree2::Punct(punct) => {
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
							throw_sg_err! {
								return [punct.span()]: "", #e_group_str, " was expected."
							}
						}

						TreeResult::Ok(optm_punct)
					}

					_ => {
						let e_group_str = make_endroup_str(endgroup);
						throw_sg_err! {
							return [m_punct.span()]: "", #e_group_str, " was expected."
						}
					}
				}
			} else {
				let e_group_str = make_endroup_str(endgroup);
				throw_sg_err! {
					return [group.span()]: "", #e_group_str, " was expected."
				}
			}
		}
		Delimiter::Brace => TreeResult::Ok(None), // `{ ... }`, ok
		Delimiter::Bracket | Delimiter::None => {
			throw_sg_err! {
				return [group.span()]: "Unsupported group type."
			}
		}
	}
}

/// A small function that mimics the incomplete behavior of stringify for groups.
pub fn group_stringify_with_fns<R>(
	group: &'_ Group,
	next: impl FnOnce(String) -> R,
	empty: impl FnOnce() -> R,
	err: impl FnOnce(TokenStream2) -> R,
) -> R {
	let mut result = String::new();

	let iter = group.stream().into_iter();
	for tt in iter {
		if let TreeResult::Err(e) = __g_stringify(tt, &mut result) {
			return err(e);
		}
	}

	if result.is_empty() {
		return empty();
	}

	next(result)
}

fn __g_stringify(tt: TokenTree2, w: &mut impl Write) -> TreeResult<()> {
	/*
		TODO, Not fully covered by tests.
	*/
	match tt {
		TokenTree2::Group(group) => {
			let iter = group.stream().into_iter();
			for tt in iter {
				tq!(__g_stringify(tt, w));
			}
		}
		TokenTree2::Ident(i) => {
			if let Err(e) = w.write_str(&i.to_string()) {
				let debug = format!("{e:?}");
				throw_sg_err! {
					return [i.span()]: "Ident, ", #debug
				}
			}
		}
		TokenTree2::Punct(p) => {
			if let Err(e) = w.write_char(p.as_char()) {
				let debug = format!("{e:?}");
				throw_sg_err! {
					return [p.span()]: "Punct, ", #debug
				}
			}
		}
		TokenTree2::Literal(l) => {
			return ExprLit::try_new_with_fns(
				&l.to_string(),
				|sspath| match w.write_str(sspath) {
					Ok(..) => TreeResult::Ok(()),
					Err(e) => {
						let debug = format!("{e:?}");
						throw_sg_err! {
							return [l.span()]: "Literal, ", #debug
						}
					}
				},
				|e| {
					let span = l.span();
					let debug = e.into_tt_err(span);

					throw_sg_err! {
						return [span]: "Literal, ", #debug
					}
				},
			);
		}
	}

	TreeResult::Ok(())
}
