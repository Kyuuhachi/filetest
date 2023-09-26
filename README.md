A proc macro for making test cases from a corpus of files, intended for parsing-related tests.

```rust
#[filetest::filetest("examples/files/*")]
fn test_file(path: &std::path::Path, bytes: &[u8], text: &str) {
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert_eq!(bytes, text.as_bytes());
}
```

The function can have any combination of the three arguments shown above[^footnote]: note that
they are identified by name, not by type.

This macro requires the `proc_macro_span` unstable feature, in order to support relative paths.

[^footnote]: All are `'static`.
