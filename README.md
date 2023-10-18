A proc macro for making test cases from a corpus of files, intended for parsing-related tests.

```rust
#[filetest::filetest("examples/files/*")]
fn test_file(path: &std::path::Path, bytes: &[u8], text: &str) {
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert_eq!(bytes, text.as_bytes());
}
```

This crate requires the `proc_macro_span` unstable feature, in order to support relative paths.

# Arguments
Arguments passed to the function are identified by name, not by type. All references are `'static`.
Currently, the following three arguments are supported:

| Name | Type | Content |
|-|-|-|
| `path`  | `&T where str: AsRef<T>`[^path] | Absolute path to the file |
| `bytes` | `&[u8]` | File contents, as seen by `include_bytes!()` |
| `text` | `&str` | File contents, as seen by `include_str!()` |

[^path]: This includes `str`, `std::path::Path`, and `camino::Utf8Path`.
