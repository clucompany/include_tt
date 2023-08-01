
use include_tt::include_tt;

#[test]
fn test_empty_tt() {
	include_tt! {
		let tt = concat!(
			// File contains: ``
			#include!("./tests/empty.tt") "Empty"
		);
		// File contains: ``
		let str = #include_str!("./tests/empty.tt");
		// File contains: ``
		let arr = #include_arr!("./tests/empty.tt");
		
		// empty path
		let tt2 = concat!(
			#include!() "Empty"
		);
		// empty path
		let str2 = #include_str!();
		// empty path
		let arr2 = #include_arr!();
	}
	
	assert_eq!(tt, "Empty");
	assert_eq!(tt2, "Empty");
	
	assert_eq!(str, "");
	assert_eq!(str2, "");
	
	assert_eq!(arr, &[]);
	assert_eq!(arr2, &[]);
}
