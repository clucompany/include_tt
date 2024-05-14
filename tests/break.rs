use include_tt::include_tt;

#[test]
fn test_empty_tt() {
	let str = include_tt! {
		#include!("./tests/empty.tt")
		/*
			The use of this keyword leaves the search for occurrences
			needed for replacement.
		*/
		#break_search_macro;

		stringify!(#include!("./tests/empty.tt"))
	};

	assert_eq!(str, "#include!(\"./tests/empty.tt\")");
	assert_eq!(
		stringify!(#include!("./tests/empty.tt")),
		"#include!(\"./tests/empty.tt\")"
	);
}
