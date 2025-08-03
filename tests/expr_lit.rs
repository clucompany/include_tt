use include_tt::inject;

#[test]
fn test_expr_lit() {
	let str = inject! {
		// File contains: `"123\"test"`
		#tt("./tests/expr_lit.tt")
		#break;
	};

	assert_eq!(str, "123\"test");
}
