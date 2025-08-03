use crate::{TreeResult, exprs::literal::ExprLit, throw_sg_err, trees::tq};
use alloc::{
	fmt::Write,
	format,
	string::{String, ToString},
};
use proc_macro2::{TokenStream as TokenStream2, TokenTree as TokenTree2};

/// A small function that mimics the incomplete behavior of stringify for stream.
pub fn stream_stringify_with_fns<R>(
	stream: TokenStream2,
	next: impl FnOnce(String) -> R,
	empty: impl FnOnce() -> R,
	err: impl FnOnce(TokenStream2) -> R,
) -> R {
	let mut result = String::new();

	let iter = stream.into_iter();
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
