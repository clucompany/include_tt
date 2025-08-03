use include_tt::inject;

#[test]
fn test_expr_lit() {
	let str = inject! {
		// File contains: `"123\"test"`
		#include_and_break!("./tests/expr_lit.tt")
	};

	assert_eq!(str, "123\"test");
}
