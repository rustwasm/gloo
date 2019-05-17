//! Provides wrappers for working with `Blob`s and `File`s from JavaScript.
//!
//! A `File` is just a `Blob` with some extra data: a name and a last modified time.
//!
//! In the File API, `Blob`s are opaque objects that lazily fetch their contained data when
//! asked. This allows a `Blob` to represent some resource that isn't completely available, for
//! example a WebSocket message that is being received or a file that needs to be read from disk.
//!
//! You can asynchronously access the contents of the `Blob` through callbacks,
//! but that is rather inconvenient, so this crate provides some functions which
//! return a `Future` instead.

mod blob;
mod file_list;
mod file_reader;

pub use blob::*;
pub use file_list::*;
pub use file_reader::*;
