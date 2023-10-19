use std::cmp;

#[derive(Debug, Clone)]
pub struct FixedStr<const N: usize> {
  len: usize,
  data: [u8; N],
}

impl<const N: usize> Default for FixedStr<N> {
  fn default() -> Self {
    let data: [u8; N] = [0; N];
    Self { len: 0, data }
  }
}

impl<const N: usize> FixedStr<N> {
  pub fn new(src: &str) -> Self {
    let data: [u8; N] = [0; N];
    let mut fs = Self { len: 0, data };
    fs.set(src);
    fs
  }

  pub fn set(&mut self, src: &str) {
    let max_len = cmp::min(N, src.len());
    for (i, c) in src.char_indices() {
      if i >= max_len {
        break;
      }
      self.data[i] = c as u8;
    }
    self.len = max_len
  }
}

impl<const N: usize> From<&FixedStr<N>> for String {
  fn from(value: &FixedStr<N>) -> Self {
    let raw = &value.data[..value.len];
    let res = String::from_utf8(raw.to_vec());
    match res {
      Ok(s) => s,
      Err(_) => String::new(),
    }
  }
}

impl<const N: usize> From<&str> for FixedStr<N> {
  fn from(value: &str) -> Self {
    FixedStr::new(value)
  }
}
