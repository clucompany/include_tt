
use include_tt::*;

#[test]
fn test_fix_unktoken() {
	#[allow(unused_assignments)]
	
	let mut b = 20;
	include_tt! {
		#include_and_fix_unknown_start_token!("./tests/invalid_token.tt")
	}
	
	assert_eq!(b, 30);
}
