use include_tt::inject;
use std::fmt::Write;
fn main() {
	let mut buf = String::new();

	inject! {
		write!(
			&mut buf,
			"Welcome, {}. Your score is {}!",
			#tt("examples/name.tt"),			// `"Ferris"`
			#tt("examples/" "score" ".tt")	// `100500`
		).unwrap();
	}

	assert_eq!(buf, "Welcome, Ferris. Your score is 100500!");
}
