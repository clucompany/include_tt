use include_tt::inject;

#[test]
fn test_empty_tt() {
	let str = inject! {
		#tt("./tests/empty.tt")
		/*
			The use of this keyword leaves the search for occurrences
			needed for replacement.
		*/
		#break;

		stringify!(#include!("./tests/empty.tt"))
	};

	assert_eq!(str, "#include!(\"./tests/empty.tt\")");
	assert_eq!(
		stringify!(#include!("./tests/empty.tt")),
		"#include!(\"./tests/empty.tt\")"
	);
}
