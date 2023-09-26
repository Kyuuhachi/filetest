#![feature(proc_macro_span)]

//! A proc macro for making test cases from a corpus of files, intended for parsing-related tests.
//!
//!```rust
//!#[filetest::filetest("../examples/files/*")]
//!fn test_file(path: &std::path::Path, bytes: &[u8], text: &str) {
//!    assert_eq!(std::fs::read(path).unwrap(), bytes);
//!    assert_eq!(bytes, text.as_bytes());
//!}
//!```
//!
//! The function can have any combination of the three arguments shown above[^footnote]: note that
//! they are identified by name, not by type.
//!
//! This macro requires the `proc_macro_span` unstable feature, in order to support relative paths.
//!
//![^footnote]: All are `'static`.

use proc_macro2::{Span, TokenStream};
use syn::{Ident, ItemFn, LitStr, Error, Result, FnArg, Pat};
use syn::spanned::Spanned;
use proc_macro_error::{proc_macro_error, Diagnostic, Level};
use quote::quote;

/// The macro in question.
///
/// See crate docs for details.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn filetest(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut output = process(attr.into(), item.clone().into())
		.map_or(proc_macro::TokenStream::new(), proc_macro::TokenStream::from);
	output.extend(item);
	output
}

#[derive(Debug, Clone)]
enum Arg {
	Path,
	Bytes,
	Text,
	Illegal(Error),
}

impl Arg {
	fn from_fnarg(arg: &FnArg) -> Arg {
		if let FnArg::Typed(arg) = arg {
			if let Pat::Ident(pat) = &*arg.pat {
				if pat.ident == "path" { return Arg::Path }
				if pat.ident == "bytes" { return Arg::Bytes }
				if pat.ident == "text" { return Arg::Text }
			}
		}
		Arg::Illegal(Error::new(arg.span(), "invalid argument for filetest"))
	}

	fn to_tokens(&self, rel: &LitStr, abs: &LitStr) -> TokenStream {
		match self {
			Arg::Path => quote! { ::std::path::Path::new(#abs) },
			Arg::Bytes => quote! { ::core::include_bytes!(#rel) },
			Arg::Text => quote! { ::core::include_str!(#rel) },
			Arg::Illegal(e) => e.to_compile_error(),
		}
	}
}

fn process(attr: TokenStream, item: TokenStream) -> Option<TokenStream> {
	let path_lit = syn::parse2::<LitStr>(attr).emit();
	let item_fn = syn::parse2::<ItemFn>(item).emit();

	let item_fn = item_fn?;

	let fn_name = &item_fn.sig.ident;
	let args: Vec<Arg> = item_fn.sig.inputs.iter()
		.map(Arg::from_fnarg)
		.collect();

	if args.is_empty() {
		Error::new(Span::call_site(), "no args").emit();
	}

	let path_lit = path_lit?;

	let mut out = TokenStream::new();

	let file_path = path_lit.span().unwrap().source_file().path();
	let file_path = file_path.parent().unwrap().canonicalize().unwrap();
	let glob_path = glob::Pattern::escape(&file_path.display().to_string());
	let glob_path = format!("{glob_path}/{}", path_lit.value());

	for path in glob::glob(&glob_path)
		.map_err(|e| Error::new(path_lit.span(), e.to_string()))
		.emit()?
	{
		if let Some(path) = path
			.map_err(|e| Error::new(path_lit.span(), e.to_string()))
			.emit()
		{
			let rel = LitStr::new(&path.strip_prefix(&file_path).unwrap().display().to_string(), path_lit.span());
			let abs = LitStr::new(&path.display().to_string(), path_lit.span());

			let test_name = test_name(&path);
			let call_args = args.iter().map(|a| a.to_tokens(&rel, &abs)).collect::<Vec<_>>();
			let ret = &item_fn.sig.output;
			out.extend(quote! {
				#[test]
				fn #test_name() #ret {
					super::#fn_name(#(#call_args),*)
				}
			})
		}
	}

	if out.is_empty() {
		Error::new(Span::call_site(), "no files found").emit();
	}

	Some(quote! {
		mod #fn_name {
			#out
		}
	})
}

fn test_name(path: &std::path::Path) -> Ident {
	let name = path.file_name().unwrap().to_str().unwrap();
	let mut name = name.chars().map(|a| {
		if unicode_ident::is_xid_continue(a) {
			a
		} else {
			'_'
		}
	}).collect::<String>();
	if !name.chars().next().is_some_and(unicode_ident::is_xid_start) {
		name.insert(0, '_')
	}
	Ident::new(&name, Span::call_site())
}

trait Emit {
	type T;
	fn emit(self) -> Self::T;
}
impl Emit for Error {
	type T = ();
	fn emit(self) {
		Diagnostic::spanned(self.span(), Level::Error, self.to_string()).emit();
	}
}
impl<T> Emit for Result<T> {
	type T = Option<T>;
	fn emit(self) -> Option<T> {
		match self {
			Ok(v) => Some(v),
			Err(e) => { e.emit(); None }
		}
	}
}
