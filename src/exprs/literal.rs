
use std::{ops::Deref, fmt::{Debug, Display}};
use proc_macro2::{Span, TokenStream as TokenStream2};
use crate::sg_err;

/// The actual literal expression, written as "./test".
#[repr(transparent)]
pub struct ExprLit {
	data: str
}

impl PartialEq<ExprLit> for ExprLit {
	#[inline(always)]
	fn eq(&self, other: &ExprLit) -> bool {
		PartialEq::eq(self.as_str(), other.as_str())
	}
}

impl PartialEq<str> for ExprLit {
	#[inline(always)]
	fn eq(&self, other: &str) -> bool {
		PartialEq::eq(self.as_str(), other)
	}
}

/// Errors received in case of a 
/// literal expression parsing error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprLitTryNewErr {
	/// More characters were expected to be parsed.
	ExpLen {
		current: usize,
		exp: usize,
	},
	/// A double closing of quotes was expected.
	ExpQuotes,
}

impl ExprLitTryNewErr {
	/// Convert an error to a syntax tree.
	#[inline]
	pub fn into_tt_err(self, span: Span) -> TokenStream2 {
		match self {
			Self::ExpLen { current, exp } => sg_err! {
				[span]: "More char expected, current: ", #current, "exp: {}", #exp, "."
			},
			Self::ExpQuotes => sg_err! {
				[span]: "Double quotes were expected."
			}
		}
	}
}

impl Deref for ExprLit {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Debug for ExprLit {
	#[inline(always)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self as &str, f)
	}
}

impl Display for ExprLit {
	#[inline(always)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self as &str, f)
	}
}

impl ExprLit {
	/// Creating `ExprLit` without clipping.
	#[inline]
	const fn __new<'a>(a: &'a str) -> &'a ExprLit {
		// It is safe, there is no other way than to "transmute", "box" 
		// to create a dimensionless structure.
		unsafe { &*(a as *const _ as *const ExprLit) }
	}
	
	/// Create an `ExprLit` from the expression `"test"` and return it.
	#[allow(dead_code)]
	#[inline]
	pub fn try_new<'a>(a: &'a str) -> Result<&'a ExprLit, ExprLitTryNewErr> {
		Self::try_new_fn(
			a,
			|ok| Ok(ok),
			|e| Err(e)
		)
	}
	
	/// Create an `ExprLit` from the expression `"test"` and return it.
	pub fn try_new_fn<'a, R>(a: &'a str, next: impl FnOnce(&'a ExprLit) -> R, err: impl FnOnce(ExprLitTryNewErr) -> R) -> R {
		let a_array = a.as_bytes();
		
		let len = a_array.len();
		if len < 2 {
			return err(ExprLitTryNewErr::ExpLen {
				current: len,
				exp: 2,
			});
		}
		
		if let (Some(b'"'), Some(b'"')) = (a_array.get(0), a_array.get(len-1)) {} else {
			return err(ExprLitTryNewErr::ExpQuotes);
		}

		next(
			Self::__new(&a[1..len-1])
		)
	}
	
	#[inline(always)]
	pub const fn as_str(&self) -> &str {
		&self.data
	}
}

#[cfg(test)]
#[test]
fn test_literal() {
	assert_eq!(
		ExprLit::try_new(""),
		Err(ExprLitTryNewErr::ExpLen { current: 0, exp: 2 })
	);
	assert_eq!(
		ExprLit::try_new("\""),
		Err(ExprLitTryNewErr::ExpLen { current: 1, exp: 2 })
	);
	assert_eq!(
		ExprLit::try_new("\"\""),
		Ok(ExprLit::__new("")),
	);
}
