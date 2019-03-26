use web_sys::{File as InternalFile, FileList as InternalFileList};

pub struct FileList {
  internal: InternalFileList,
  length: usize,
}

impl FileList {
  pub fn from_raw(internal: InternalFileList) -> FileList {
    let length = internal.length() as usize;
    FileList { internal, length }
  }

  pub fn get(&self, index: usize) -> Option<File> {
    self.internal.get(index as u32).map(File::from_raw)
  }

  pub fn len(&self) -> usize {
    self.length
  }

  pub fn iter(&self) -> FileListIter {
    FileListIter {
      file_list: self,
      current: 0,
    }
  }

  pub fn into_vec(self) -> Vec<File> {
    self.iter().collect()
  }
}

pub struct FileListIter<'a> {
  file_list: &'a FileList,
  current: usize,
}

impl<'a> Iterator for FileListIter<'a> {
  type Item = File;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current >= self.file_list.len() {
      return None;
    }
    let file = self.file_list.get(self.current);
    self.current += 1;

    assert!(file.is_some());

    file
  }
}

pub struct File {
  internal: InternalFile,
}

impl File {
  fn from_raw(internal: InternalFile) -> File {
    File { internal }
  }
}
