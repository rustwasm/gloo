## Summary

Add [`FileReaderSync`](https://developer.mozilla.org/en-US/docs/Web/API/FileReaderSync) API to `gloo-file`

## Motivation

To enable synchronous File reading from Web Workers.

## Detailed Explanation

Unlike the callback-based FileReader, this API is not callback-driven, which means that we can make it simpler, as:
- A RAII style struct for automatically cancelling the read is not required, which means we don't need a struct.

```rust
mod FileReaderSync {
    pub fn readAsArrayBuffer (blob: &Blob) -> Result<ArrayBuffer, FileReadError>
    // ... similar method signatures to FileReader, without the callbacks
}
```

## Drawbacks, Rationale, and Alternatives

Can consider transforming `FileReaderSync` into a _trait_ which is implemented on `Blobs` and such, which makes the API prettier, but less clear?
```rust
use gloo_file::FileReaderSync;

let file = get_file().readAsArrayBuffer();
let data = Uint8Array::new(file);
```

Prior art:
- [FileReaderSync MDN docs](https://developer.mozilla.org/en-US/docs/Web/API/FileReaderSync)
- [FileReader implementation in Gloo](https://github.com/rustwasm/gloo/blob/master/crates/file/src/file_reader.rs)

## Unresolved Questions

Might look into adding a few more error-types to `FileReadError`, which are specific to particular types of reads, such as `EncodingError` present in [`readAsDataURL`](https://developer.mozilla.org/en-US/docs/Web/API/FileReaderSync/readAsDataURL).

Don't know whether they are in the scope of this PR.
