
#[macro_export]
macro_rules! __test_custom_path {
	[ @$n: ident($const_t: ident) : [$t1:tt $t2:tt $t3:tt $t4:tt $t5:tt $t6:tt]; ] => {
		include_tt::include_tt! {
			#[allow(dead_code)]
			#[allow(non_upper_case_globals)]
			pub mod ttest {
				pub const a: usize = 0;
				pub const b: usize = 10;
				
				pub const $const_t: (usize, usize) = (#include!([$t1 $t2 $t3 $t4 $t5 $t6]));
			}
		}
	};
}


fn main() {
	__test_custom_path! {
		@ttest(T): [for_examples / "full" . t 't'];
	}
	assert_eq!(ttest::T, (0, 10));
}
