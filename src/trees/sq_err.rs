
/// A small macro that allows you to prepare 
/// an error tree and throw it to the user.
macro_rules! sg_err {
	// return macro `compile_error!`.
	[ return $($tt:tt)* ] => {
		return sg_err! {
			$($tt)*
		}
	};
	
	// break macro `compile_error!` with a concatenator.
	[ break $($tt:tt)* ] => {
		break sg_err! {
			$($tt)*
		}
	};
	
	// macro `compile_error!`.
	[ [$span:expr]: $err:expr $(,)? ] => {
		quote::quote_spanned! {
			$span =>
			compile_error!($err);
		}.into()
	};
	
	// macro `compile_error!` with a concatenator.
	[ [$span:expr]: $($err:tt)+ ] => {
		quote::quote_spanned! {
			$span =>
			compile_error!(
				concat!( $($err)+ )
			);
		}.into()
	};
}
