use include_tt::inject;

#[test]
fn test_fix_unktoken() {
	#[allow(unused_assignments)] // bug?
	let mut b = 20;
	assert_eq!("\\".len(), 1);
	inject! {
		// File contains: `\ b = 30; \`
		#ctt!("./tests/invalid_token.tt")
	}

	assert_eq!(b, 30);
}
