use include_tt::inject;

#[test]
fn test_empty_tt() {
	inject! {
		let tt = concat!(
			// File contains: ``
			#tt("./tests/empty.tt") "Empty"
		);
		// File contains: ``
		let str = #str("./tests/empty.tt");
		// File contains: ``
		let arr = #arr("./tests/empty.tt");

		// empty path
		let tt2 = concat!(
			#tt() "Empty"
		);
		// empty path
		let str2 = #str();
		// empty path
		let arr2 = #arr();
	}

	assert_eq!(tt, "Empty");
	assert_eq!(tt2, "Empty");

	assert_eq!(str, "");
	assert_eq!(str2, "");

	assert_eq!(arr, &[]);
	assert_eq!(arr2, &[]);
}
