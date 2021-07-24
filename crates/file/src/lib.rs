//!
//! Working with files and blobs on the Web.
//!
//! These APIs come in two flavors:
//!
//! 1. a callback style (that more directly mimics the JavaScript APIs), and
//! 2. a `Future` API.

mod blob;
mod file_list;
mod file_reader;
mod sync_file_reader;

pub use blob::*;
pub use file_list::*;
pub use file_reader::*;
pub use sync_file_reader::*;