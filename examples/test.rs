#[filetest::filetest("files/*")]
fn test_file(
	path: &std::path::Path,
	bytes: &[u8],
	text: &str,
) -> std::io::Result<()> {
	assert_eq!(std::fs::read(path)?, bytes);
	assert_eq!(bytes, text.as_bytes());
	Ok(())
}

// This creates tests `test_file::example_txt` and `test_file::file_rs`

fn main() {}
