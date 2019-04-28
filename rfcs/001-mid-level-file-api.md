# Mid-Level File Api

## Summary

This is a mid-level API wrapper around the various File related JavaScript apis. The possibility for a higher level API is left open.

The mid-level file API aims to implement File I/O on top of the raw JS apis found here: https://w3c.github.io/FileAPI/ which includes listing files, creating files, and reading files.

## The API

First we have `FileList` which is simply a list of `File`s. This is an opaque struct that offers a way to iterate over `Files` and get individual files by index:

```rust
#[derive(Debug, Clone)]
struct FileList { ... }

impl FileList {
  fn get(index: u32) -> Option<File> { ... }

  fn len(&self) -> u32 { ... }

  fn iter(&self) -> FileListIter { ... }

  fn as_raw(&self) -> &web_sys::FileList { ... }
}

impl AsRef<web_sys::FileList> for FileList { ... }
impl From<web_sys::FileList> for FileList { ... }
impl Index<usize> for FileList {
    type Output = File;
    fn index(&self, index: usize) -> &Self::Output { ... }
}
```

There is no `FileList::new` since creating a raw `web_sys::FileList` is not possible without going through a `web_sys::HtmlInputElement`.

Next we the trait `BlobLike`
```rust
trait BlobLike {
    fn size(&self) -> u64 { ... }

    // the mime crate and mimes from blobs both conform to rfc6838
    #[cfg(feature = "mime")]
    fn mime_type(&self) -> Result<mime::Mime, mime::FromStrError> { ... }

    fn raw_mime_type(&self) -> String { ... }

    fn as_raw(&self) -> &web_sys::Blob;

    fn slice(&self, start: u64, end: u64) -> Self 
}
```
There are two structs that implement this trait: `Blob` and `File`.

```rust
#[derive(Debug, Clone)]
struct Blob { ... }

impl Blob {
  fn new<T>(contents: T) -> Blob
    where
        T: std::convert::Into<BlobContents> // We'll look at BlobContents below
    { ... }

  fn new_with_options<T>(contents: T, mime_type: String) -> Blob
    where
        T: std::convert::Into<BlobContents>
    { ... }
}

impl From<web_sys::Blob> for Blob { ... }

impl BlobLike for Blob { ... }

#[derive(Debug, Clone)]
pub struct File { ... }

impl File {
  fn new<T>(
        name: String,
        contents: T,
  ) -> File
    where
        T: std::convert::Into<BlobContents>,
    { ... }

  fn new_with_options<T>(
        name: String,
        contents: T,
        mime_type: Option<String>,
        last_modified_date: Option<u64>,
  ) -> File
    where
        T: std::convert::Into<BlobContents>,
    { ... }

  fn name(&self) -> String { ... }

  fn last_modified_since_epoch(&self) -> Duration { ... }

  fn as_blob(&self) -> Blob { ... }
}

impl BlobLike for File { ... }
```

`BlobContents` is simply a new-type around `wasm_bindgen::JsValue`s that can be used as the content of `Blob`s and `File`s:

```rust
#[derive(Debug, Clone)]
pub struct BlobContents {
    inner: wasm_bindgen::JsValue,
}
```

There are there conversions from types into `BlobContents` only for the types that make sense:

```rust
impl std::convert::Into<BlobContents> for &str
impl std::convert::Into<BlobContents> for &[u8]
impl std::convert::Into<BlobContents> for Blob
impl std::convert::Into<BlobContents> for js_sys::ArrayBuffer
```

Lastly there's the `FileReader` which allows reading from BlobLike objects. We'll have two implementations of this, one based on callbacks and the other based on futures.

The callbacks implementation has three categories of callbacks: start, progress and read. Start and progress callbacks are directly related to the `onloadstart` and `onprogress` callbacks on `web_sys::FileReader`. The read variety of callbacks, are a combination of `onload`, `onloadend`, `onloaderror`, and `onabort`. The callback receives a result which is an error if the underlying read was aborted or errored.

The futures implementation likewise exposes success, error, and abort through the fact that futures are `Result`-like. Progress events are exposed as a stream. In the future, we may expose the entire lifecycle of a read through a stream.

```rust
mod callbacks {
  #[derive(Debug)]
  pub struct FileReader { ... }

  impl FileReader {
    fn new() -> FileReader { ... }

    fn read_as_string<F>(self, blob: &impl BlobLike, callback: F)
        where F: FnOnce(Result<String, FileReadError>) { ... };

    fn read_as_data_url<F>(self, blob: &impl BlobLike, callback: F)
        where F: FnOnce(Result<String, FileReadError>) { ... };

    fn read_as_array_buffer<F>(self, blob: &impl BlobLike, callback: F)
        where F: FnOnce(Result<&web_sys::ArrayBuffer, FileReadError>) { ... };

    fn on_progress<F>(&mut self, callback: F)
      where F: FnMut(ProgressEvent) + 'static { ... }

    fn on_load_start<F>(&mut self, callback: F)
      where F: FnOnce(LoadStartEvent) + 'static { ... }
  }
}

mod futures {
  #[derive(Debug)]
  pub struct FileReader { ... }

  impl FileReader {
    fn new() -> FileReader { ... }

    fn read_as_string(self, blob: &impl BlobLike) -> ReadAsString { ... }

    fn read_as_data_url(self, blob: &impl BlobLike) -> ReadAsDataUrl { ... }

    fn read_as_array_buffer(self, blob: &impl BlobLike) -> ReadAsArrayBuffer { ... }

    fn on_progress(&self) -> OnProgressStream { }

    fn on_load_start(&self) -> OnLoadStartFuture { }
  }

  pub struct ReadAsString { ... }
  impl Future for ReadAsString {
      type Item = String;
      type Error = FileReadError;
      ...
  }

  // Make sure that dropping the Future properly aborts the reading
  impl std::ops::Drop for ReadAsString {
      fn drop(&mut self) {
          if self.inner.ready_state() < 2 {
              self.inner.abort();
          }
      }
  }
}
```
