use chumsky::Parser;
use insta::{assert_debug_snapshot, glob};
use std::fs;

#[test]
fn test_local() {
	glob!("fixtures/raw/ok/*.txt", |path| {
		let input = fs::read_to_string(path).unwrap();

		let result = simai::parse::simai().parse(&input);
		assert!(
			!result.has_errors(),
			"failed to parse {}: {:?}",
			path.display(),
			result.errors().collect::<Vec<_>>()
		);

		let output = result.output();
		let output = output.unwrap_or_else(|| panic!("failed to parse: {}", path.display()));

		assert_debug_snapshot!(output);
	});

	glob!("fixtures/raw/err/*.txt", |path| {
		let input = fs::read_to_string(path).unwrap();

		let result = simai::parse::simai().parse(&input);

		let output = result.output();
		let errors = result.errors().collect::<Vec<_>>();

		assert_debug_snapshot!((output, errors));
	});
}
