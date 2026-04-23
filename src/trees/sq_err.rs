/// A small macro that allows you to create an error tree and either
/// return it to the calling function or break it in a loop
macro_rules! sq_err {
	// return macro `compile_error!`.
	[ return $($tt:tt)* ] => {
		return sq_err! {
			$($tt)*
		}
	};

	// break macro `compile_error!` with a concatenator.
	[ break $($tt:tt)* ] => {
		break sq_err! {
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
	[ [$span:expr]: $($err:tt)+ ] => {{
		$crate::trees::__sq_err_format!(
			@let_block: in[ $($err)+ ]
		);

		$crate::trees::__sq_err_format!(
			quote::quote_spanned! {
				$span =>
				compile_error!(
					concat!( in[ $($err)+ ] args[] )
				)
			}
		)
	}};
}

macro_rules! __sq_err_format {
	[
		@let_block:
		in[ # {$n:ident :?} $($all:tt)* ]
	] => {
		let $n = format!("{:?}", $n);

		$crate::trees::__sq_err_format! {
			@let_block:
			in [ $($all)* ]
		}
	};

	[
		@let_block:
		in[ # {$n:ident} $($all:tt)* ]
	] => {
		let $n = format!("{}", $n);

		$crate::trees::__sq_err_format! {
			@let_block:
			in [ $($all)* ]
		}
	};

	[
		@let_block:
		in[ $_t:tt $($all:tt)* ]
	] => {
		$crate::trees::__sq_err_format! {
			@let_block:
			in [ $($all)* ]
		}
	};

	[ // END
		@let_block:
		in[]
	] => {};


	[ // {:?}
		quote::quote_spanned! {
			$span:expr =>
			compile_error!(concat!(
				in[ # {$n:ident :?} $($all:tt)* ]
				args[ $($args:tt)* ]
			))
		}
	] => {
		$crate::trees::__sq_err_format! {
			quote::quote_spanned! {
				$span =>
				compile_error!(concat!(
					in[ $($all)* ]
					args[ $($args)* # $n ]
				))
			}
		}
	};

	[ // {}
		quote::quote_spanned! {
			$span:expr =>
			compile_error!(concat!(
				in[ # {$n:ident} $($all:tt)* ]
				args[ $($args:tt)* ]
			))
		}
	] => {
		$crate::trees::__sq_err_format! {
			quote::quote_spanned! {
				$span =>
				compile_error!(concat!(
					in[ $($all)* ]
					args[ $($args)* # $n ]
				))
			}
		}
	};

	[
		quote::quote_spanned! {
			$span:expr =>
			compile_error!(concat!(
				in[ $t:tt $($all:tt)* ]
				args[ $($args:tt)* ]
			))
		}
	] => {
		$crate::trees::__sq_err_format! {
			quote::quote_spanned! {
				$span =>
				compile_error!(concat!(
					in[ $($all)* ]
					args[ $($args)* $t ]
				))
			}
		}
	};

	[ // END
		quote::quote_spanned! {
			$span:expr =>
			compile_error!(concat!(
				in[ ]
				args[ $($args:tt)* ]
			))
		}
	] => {
		quote::quote_spanned! {
			$span =>
			compile_error!(
				concat!( $($args)* )
			);
		}.into()
	};
}
