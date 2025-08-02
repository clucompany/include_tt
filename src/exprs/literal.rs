use crate::throw_sg_err;
use core::{
	fmt::{Debug, Display},
	ops::Deref,
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::ffi::OsStr;

/// The actual literal expression, written as "./test".
#[repr(transparent)]
pub struct ExprLit(str);

impl PartialEq<ExprLit> for ExprLit {
	#[inline]
	fn eq(&self, other: &ExprLit) -> bool {
		PartialEq::eq(self.as_str(), other.as_str())
	}
}

impl PartialEq<str> for ExprLit {
	#[inline]
	fn eq(&self, other: &str) -> bool {
		PartialEq::eq(self.as_str(), other)
	}
}

impl AsRef<OsStr> for ExprLit {
	#[inline]
	fn as_ref(&self) -> &OsStr {
		AsRef::as_ref(self.as_str())
	}
}

/// Errors received in case of a
/// literal expression parsing error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprLitTryNewErr {
	/// More characters were expected to be parsed.
	ExpLen { current: usize, exp: usize },

	/// A double closing of quotes was expected.
	ExpQuotes,
}

impl ExprLitTryNewErr {
	#[inline]
	pub const fn expr_len(current: usize, exp: usize) -> Self {
		Self::ExpLen { current, exp }
	}

	/// Convert an error to a syntax tree.
	#[inline]
	pub fn into_tt_err(self, span: Span) -> TokenStream2 {
		match self {
			Self::ExpLen { current, exp } => throw_sg_err! {
				[span]: "More char expected, current: ", #current, "exp: {}", #exp, "."
			},
			Self::ExpQuotes => throw_sg_err! {
				[span]: "Double quotes were expected."
			},
		}
	}
}

impl Deref for ExprLit {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Debug for ExprLit {
	#[inline]
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		Debug::fmt(self as &str, f)
	}
}

impl Display for ExprLit {
	#[inline]
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		Display::fmt(self as &str, f)
	}
}

impl ExprLit {
	/// Creating `ExprLit` without clipping.
	#[inline]
	const fn __new(a: &str) -> &ExprLit {
		// It is safe, there is no other way than to "transmute", "box"
		// to create a dimensionless structure.
		unsafe { &*(a as *const _ as *const ExprLit) }
	}

	/// Creating `ExprLit` without clipping.
	#[inline]
	pub const unsafe fn new_unchecked(a: &str) -> &Self {
		Self::__new(a)
	}

	/// Create an `ExprLit` from the expression `"test"` and return it.
	#[allow(dead_code)]
	#[inline]
	pub fn try_new(a: &str) -> Result<&ExprLit, ExprLitTryNewErr> {
		Self::try_new_with_fns(a, Ok, Err)
	}

	/// Create an `ExprLit` from the expression `"test"` and return it.
	pub fn try_new_with_fns<'a, R>(
		a: &'a str,
		next: impl FnOnce(&'a ExprLit) -> R,
		err: impl FnOnce(ExprLitTryNewErr) -> R,
	) -> R {
		let a_array = a.as_bytes();

		let len = a_array.len();
		if len < 2 {
			return err(ExprLitTryNewErr::expr_len(len, 2));
		}
		debug_assert!({
			#[allow(clippy::get_first)]
			// why?, this is done to be completely analogous to an unsafe function.
			a_array.get(0).is_some()
		});
		debug_assert!({
			#[allow(clippy::get_first)]
			// why?, this is done to be completely analogous to an unsafe function.
			a_array.get(len - 1).is_some()
		});
		/*
			This is safe, the extra necessary checks are done in a separate `if` above.
		*/
		match unsafe { (a_array.get_unchecked(0), a_array.get_unchecked(len - 1)) } {
			(b'"', b'"') =>
				/* line */
				{} // GOOD,
			(b'\'', b'\'') if len != 3 => {
				/* one_symbol */
				/*
					We exclude the possibility of using `'` as more
					than one character.
				*/
				return err(ExprLitTryNewErr::expr_len(len, 3));
			}
			(b'\'', b'\'') =>
				/* line */
				{} // GOOD,
			_ => return err(ExprLitTryNewErr::ExpQuotes),
		}

		next({
			debug_assert!(a.get(1..len - 1).is_some());

			// It's safe, checks are done above (above `debug_assert`).
			let str = unsafe { a.get_unchecked(1..len - 1) };
			Self::__new(str)
		})
	}

	#[allow(dead_code)]
	#[inline]
	/// Returns `true` if self has a length of zero bytes.
	pub const fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Getting a string of actual data.
	#[inline]
	pub const fn as_str(&self) -> &str {
		&self.0
	}
}

#[cfg(test)]
#[test]
fn test_literal() {
	/*
		Checking the correct operation of ExprLit.
	*/
	assert_eq!(ExprLit::try_new(""), Err(ExprLitTryNewErr::expr_len(0, 2)));
	assert_eq!(
		ExprLit::try_new("\""),
		Err(ExprLitTryNewErr::expr_len(1, 2))
	);
	assert_eq!(ExprLit::try_new("\"\""), Ok(ExprLit::__new("")),);
	assert_eq!(ExprLit::try_new("'\\'"), Ok(ExprLit::__new("\\")),);
}
