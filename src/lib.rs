#![allow(non_snake_case)]

//! Command-line argument sorting and parsing for Rust.
//!
//! **CLASP.Rust** follows the C **CLASP** library. A flag, an option, and a
//! value are the argument kinds. Specifications supply aliases, and the
//! normative result order is flag, then option, then value.
//!
//! Parsing does not panic and does not exit the process. An empty
//! `argv`, which the C contract leaves undefined, yields an empty
//! program name and no arguments.
//!
//! # Used-tracking
//!
//! Marking queries take `&self` and store the used bit in a [`Cell`].
//! That matches the C API: the arguments pointer is const, and the
//! function still marks the argument. The trade-off is that
//! [`Argument`] is not [`Sync`], and the mark is not visible in the
//! signature. An `&mut self` API would show the mark, and it would
//! also stop the caller holding an argument reference across a query.
//!
//! [`flag_is_specified`][Arguments::flag_is_specified],
//! [`check_flag`][Arguments::check_flag],
//! [`check_all_flags`][Arguments::check_all_flags],
//! [`check_all_matching_flags`][Arguments::check_all_matching_flags],
//! [`find_flag_or_option`][Arguments::find_flag_or_option],
//! [`check_value`][Arguments::check_value], and
//! [`Argument::mark_used`] set the used bit. Report queries do not.
//!
//! # Examples
//!
//! ```
//! use clasp_rs::{
//!     parse,
//!     ParseFlags,
//!     Specification,
//! };
//!
//! let specifications = [
//!     Specification::flag(Some("-v"), Some("--verbose"), None),
//! ];
//! let argv = [
//!     "tool",
//!     "-v",
//! ];
//! let parsed = parse(ParseFlags::empty(), &argv, Some(&specifications));
//!
//! assert!(parsed.flag_is_specified("--verbose"));
//! ```
//!
//! [`Cell`]: std::cell::Cell
//! [`Sync`]: std::marker::Sync

// lib.rs : CLASP.Rust

// /////////////////////////////////////////////////////////
// crate-level feature definitions

// /////////////////////////////////////////////////////////
// crate-level feature discrimination

// /////////////////////////////////////////////////////////
// imports

mod arguments;
mod parse;
mod queries;
mod specifications;

pub use arguments::{
    Argument,
    ArgumentKind,
    Arguments,
};
pub use parse::{
    parse,
    ParseFlags,
};
pub use queries::Report;
pub use specifications::{
    Specification,
    SpecificationType,
};

// ///////////////////////////// end of file //////////////////////////// //
