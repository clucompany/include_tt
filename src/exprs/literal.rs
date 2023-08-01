
use std::{ops::Deref, fmt::{Debug, Display}, borrow::{Cow, Borrow}};
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

impl ToOwned for ExprLit {
	type Owned = String;
	
	#[inline]
	fn to_owned(&self) -> Self::Owned {
		self.as_str().to_owned()
	}
}

impl Borrow<ExprLit> for String {
	#[inline]
	fn borrow(&self) -> &ExprLit {
		unsafe { ExprLit::unchecked(self.as_str()) } // TODO
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
	
	/// Creating `ExprLit` without clipping.
	#[inline(always)]
	pub const unsafe fn unchecked(a: &str) -> &Self {
		Self::__new(a)
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
		
		debug_assert_eq!(a_array.get(0).is_some(), true);
		debug_assert_eq!(a_array.get(len -1).is_some(), true);
		/*
			This is safe, the extra necessary checks are done in a separate `if` above.
		*/
		match unsafe { (a_array.get_unchecked(0), a_array.get_unchecked(len -1)) } {
			(b'"', b'"') => /* line */ {}, // GOOD,
			(b'\'', b'\'') if len != 3 => { /* one_symbol */
				/*
					We exclude the possibility of using `'` as more 
					than one character.
				*/
				return err(
					ExprLitTryNewErr::ExpLen {
						current: len, 
						exp: 3
					}
				);
			},
			(b'\'', b'\'') => /* line */ {}, // GOOD,
			_ => return err(ExprLitTryNewErr::ExpQuotes),
		}
		
		next({
			debug_assert_eq!(a.get(1..len-1).is_some(), true);
			
			// It's safe, checks are done above (above `debug_assert`).
			let str = unsafe {
				a.get_unchecked(1..len-1)
			};
			Self::__new(str)
		})
	}
	
	/// Create an `ExprLit` from the expression `"test"` and return it.
	#[inline]
	#[deprecated(since="1.0.2", note="please use `try_new` instead")]
	pub fn try_new_search_and_autoreplaceshielding<'a>(a: &'a str) -> Result<Cow<'a, ExprLit>, ExprLitTryNewErr> {
		#[allow(deprecated)]
		Self::try_new_search_and_autoreplaceshielding_fn(
			a, 
			|a| Ok(a),
			|e| Err(e),
		)
	}
	
	/// Create an `ExprLit` from the expression `"test"` and return it.
	#[deprecated(since="1.0.2", note="please use `try_new_fn` instead")]
	pub fn try_new_search_and_autoreplaceshielding_fn<'a, R>(a: &'a str, next: impl FnOnce(Cow<'a, ExprLit>) -> R, err: impl FnOnce(ExprLitTryNewErr) -> R) -> R {
		Self::try_new_fn(
			a, 
			|exprlit| match a.find('\\') {
				None => next(Cow::Borrowed(exprlit)),
				Some(pos) => {
					let mut result = String::with_capacity(a.len());
					
					let all_str = match pos {
						0 => exprlit.as_str(),
						pos => {
							let (push_str, all_str) = exprlit.split_at(pos-1);
							result.push_str(push_str);
							
							all_str
						},
					};
					
					let mut iter = all_str.chars();
					while let Some(asymb) = iter.next() {
						if asymb == '\\' {
							match iter.next() {
								Some(asymb) => result.push(asymb),
								None => break,
							}
							
							continue;
						}
						
						result.push(asymb);
					}
					
					next(Cow::Owned(result))
				},
			},
			|e| err(e)
		)
	}
	
	#[inline(always)]
	/// Returns `true` if self has a length of zero bytes.
	pub const fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
	
	/// Getting a string of actual data.
	#[inline(always)]
	pub const fn as_str(&self) -> &str {
		&self.data
	}
}

#[cfg(test)]
#[test]
fn test_literal() {
	/*
		Checking the correct operation of ExprLit.
	*/
	
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
	assert_eq!(
		ExprLit::try_new("'\\'"),
		Ok(ExprLit::__new("\\")),
	);
}
