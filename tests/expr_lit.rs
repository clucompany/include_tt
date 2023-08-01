
use include_tt::include_tt;

#[test]
fn test_expr_lit() {
	let str = include_tt! {
		// File contains: `"123\"test"`
		#include!("./tests/expr_lit.tt")
	};
	
	assert_eq!(str, "123\"test");
}
