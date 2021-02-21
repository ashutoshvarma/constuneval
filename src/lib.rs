//! Simple `Cow` focussed serializer for generating const Rust code using Debug trait.
//!
//! ## Usage
//! In general, to embed some code(tables/struct) into crate, you have to use the build script
//! and [`include!`][include] macro. Inside the build script, you'll generate
//! some code with one of the [to_file()][to_file], [to_string()][to_string]
//! provided by `constuneval`,
//! and then include the generated file, like this:
//! ```ignore
//! include!(concat!(env!(OUT_DIR), "/file_name.rs"));
//! ```
//!
//! Also this crate provides a fork of [UnevalCow]
//! but with better serialization though Debug trait
//!
//! ## How does it work?
//!
//! To keep things simple all the formatting/serialization is done with help
//! of Debug trait. For most types such as `struct`, `enum`, `Vec`, etc it
//! works fine, but not for `Deref` like types like `Cow` as their Debug essentially
//! deref before formatting. To address this crate also provides [UnevalCow] as a
//! substitute to [std::borrow::Cow].
//!
//! Of course, we can't always directly construct the code for the desired value (more on this
//! in the [Limitations](#limitations) section below).
//!
//! ## Example
//! ```
//! use constuneval::{to_file, to_string, UnevalCow};
//! use std::fmt;
//!
//! #[derive(Debug)]
//! pub struct FftDomain<F>
//! where
//!     [F]: 'static + ToOwned + fmt::Debug,
//!     <[F] as std::borrow::ToOwned>::Owned: fmt::Debug,
//! {
//!     pub some_table: UnevalCow<'static, [UnevalCow<'static, [F]>]>,
//! }
//!
//! // some build time generated struct table
//! let fft_temp = FftDomain {
//!     some_table: UnevalCow::Owned(vec![
//!         UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
//!         UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
//!         UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
//!         UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
//!         UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
//!         UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
//!     ]),
//! };
//!
//!
//! to_file(
//!     std::path::Path::new("const_fft_tables.rs"),
//!     "FFT_TABLE",
//!     &fft_temp,
//!     Some("FftDomain<'static, i32>"),
//! )
//! .expect("Write Failed");
//! ```
//!
//! content of `const_fft_tables.rs` (after running rustfmt on it)
//! ```ignore
//! const FFT_TABLE: FftDomain<'static, i32> = FftDomain {
//!     some_table: UnevalCow::Borrowed(&[
//!         UnevalCow::Borrowed(&[1, 2, 3, 4, 5]),
//!         UnevalCow::Borrowed(&[1, 2, 3, 4, 5]),
//!         UnevalCow::Borrowed(&[1, 2, 3, 4, 5]),
//!         UnevalCow::Borrowed(&[1, 2, 3, 4, 5]),
//!         UnevalCow::Borrowed(&[1, 2, 3, 4, 5]),
//!         UnevalCow::Borrowed(&[1, 2, 3, 4, 5]),
//!     ]),
//! };
//! ```
//!
//! Now this file/code can be embed into crate using [`include!`][include] macro.
//!
//! ## Limitations
//! There are some cases when `constuneval` will be unable to generate valid code. Namely:
//! 1. This serializer is intended for use with types with well implemented Debug trait. It may not
//! work if Debug trait is producing invalid outputs.
//!
//! [include]: https://doc.rust-lang.org/stable/std/macro.include.html

use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod uneval_cow;

pub use uneval_cow::UnevalCow;

/// Obtain string with generated const Rust code.
pub fn to_string<T: fmt::Debug>(name: &str, value: &T, ty: Option<&str>) -> String {
    let type_name = ty.unwrap_or(std::any::type_name::<T>());
    return format!("const {}: {} = {:#?};", name, type_name, value);
}

/// Generate the const Rust code and write it to temporary file
///
/// When Cargo runs your crate's build task,
/// it sets the `OUT_DIR` environment variable to the path to build target directory (see
/// [Cargo reference](https://doc.rust-lang.org/cargo/reference/environment-variables.html) for more).
/// So, you can use it in two steps:
/// 1. Generate the Rust code and write it to temporary file:
/// ```ignore
/// # let value = ();
/// let path: std::path::PathBuf = [
///     std::env::var("OUT_DIR").expect("No build target path set"),
///     "file_name.rs".into()
/// ].iter().collect();
/// constuneval::to_file(path, "MYVAR", value, Some("MyType")).expect("Write failed");
/// ```
/// 2. [Include][include] the generated Rust code wherever it is needed:
/// ```ignore
/// // code here
/// include!(concat!(env!(OUT_DIR), "/file_name.rs"));
/// ```
///
/// [include]: https://doc.rust-lang.org/stable/std/macro.include.html
pub fn to_file<T: fmt::Debug>(
    target: impl AsRef<std::path::Path>,
    name: &str,
    value: &T,
    ty: Option<&str>,
) -> Result<(), io::Error> {
    let mut file = File::create(target)?;
    file.write_all(to_string(name, value, ty).as_bytes())?;
    Ok(())
}
