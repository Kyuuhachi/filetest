use std::io;
use std::path::Path;

#[filetest::filetest("files/*")]
fn test_file(
	path: &Path,
	bytes: &[u8],
	text: &str,
) -> io::Result<()> {
	assert_eq!(std::fs::read(path)?, bytes);
	assert_eq!(bytes, text.as_bytes());
	Ok(())
}

#[filetest::filetest("files/*")]
fn path_as_str(path: &str) {}

// This creates tests `test_file::example_txt` and `test_file::file_rs`

fn main() {}
