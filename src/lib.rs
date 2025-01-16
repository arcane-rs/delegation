#![expect(clippy::needless_doctest_main, reason = "readme")]
#![doc = include_str!("../README.md")]

// Not part of the public API.
#[doc(hidden)]
#[path = "macros.rs"]
pub mod __macros;

#[doc(inline)]
pub use codegen::delegate;
