use wasm_bindgen::{throw_str, throw_val};

use wasm_bindgen::{prelude::*, JsCast, UnwrapThrowExt};
use web_sys::FileReaderSync;
use crate::FileReadError;
use crate::blob::Blob;

/// Asynchronously converts `blob` into a text string and then passes it to the `callback`.
///
/// If the returned `FileReader` is dropped before the callback is called, the read will be
/// cancelled.
pub fn read_as_text (blob: &Blob) -> Result<String, FileReadError>
{
	let fr = FileReaderSync::new().unwrap_throw();
	fr.read_as_text(blob.as_ref()).map_err(
		|e| {
			throw_str(&format!("{:?}", e));

			let e= e.as_string().unwrap_throw();
			match e.as_ref () {
				"NotFoundError" => FileReadError::NotFound(e),
				"NotReadableError" => FileReadError::NotReadable(e),
				"SecurityError" => FileReadError::Security(e),
				// This branch should never be hit, so returning a less helpful error message is
				// less of an issue than pulling in `format!` code.
				_ => throw_str("unrecognised error type"),
			}
		})
}