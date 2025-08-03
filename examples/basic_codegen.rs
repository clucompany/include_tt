macro_rules! new_module {
	[ @($const_t: ident) : [ $($path:tt)* ]; ] => {
		include_tt::inject! {
			#[allow(dead_code)]
			#[allow(non_upper_case_globals)]
			pub mod my_module {
				pub const a: usize = 0;
				pub const b: usize = 10;
				
				// The `#POINT_TRACKER_FILES:` marker allows the macro to add additional 
				// instructions that tell the compiler which files to track so that it can 
				// recompile the macro if they change. This is completely optional, but without 
				// it tracking will not work.
				#POINT_TRACKER_FILES: 
				
				pub const $const_t: (usize, usize) = (#tt($($path)*));
			}
		}
	};
}

fn main() {
	// we created a module "my_module" and a constant "T" containing (a, b).
	//
	// if you need to change, for example, to (b,a) or substitute constant values,
	// we will only change the contents of the file "for_examples/full.tt"!
	new_module! {
		@(T): [examples / "full" . t 't']; // this file contains "a, b", see "for_examples/full.tt"
	}
	assert_eq!(my_module::T, (0, 10));
}
