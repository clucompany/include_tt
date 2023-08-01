
use include_tt::include_tt;

#[test]
fn test_fix_unktoken() {
	#[allow(unused_assignments)] // bug?
	
	let mut b = 20;
	assert_eq!("\\".len(), 1);
	include_tt! {
		// File contains: `\ b = 30; \`
		#include_and_fix_unknown_start_token!("./tests/invalid_token.tt")
	}
	
	assert_eq!(b, 30);
}
