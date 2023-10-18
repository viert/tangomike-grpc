use super::{entry::TrackFileEntry, error::TrackFileError, header::Header};
use chrono::{DateTime, Utc};
use std::{
  fmt::Display,
  fs::{File, OpenOptions},
  io::{Seek, SeekFrom, Write},
  mem::size_of,
  os::unix::prelude::FileExt,
  path::PathBuf,
  ptr::slice_from_raw_parts,
};

#[allow(clippy::size_of_in_element_count)]
fn to_raw<T: Sized>(obj: &T) -> Vec<u8> {
  let slice = slice_from_raw_parts(obj, size_of::<T>()) as *const [u8];
  let slice = unsafe { &*slice };
  slice.into()
}

fn from_raw<T: Sized + Clone, I: AsRef<str> + Display>(
  data: &[u8],
  ident: I,
) -> std::result::Result<T, TrackFileError> {
  if data.len() < size_of::<T>() {
    Err(TrackFileError::InsufficientDataLength(
      ident.to_string(),
      data.len(),
    ))
  } else {
    let slice = data as *const [u8] as *const T;
    let tp = unsafe { &*slice };
    Ok(tp.clone())
  }
}

pub struct TrackFile {
  flight_id: String,
  file: File,
  path: PathBuf,
}

impl TrackFile {
  pub fn new(path: PathBuf, flight_id: &str) -> Result<Self, TrackFileError> {
    let res = Self::open(path.clone(), flight_id);
    match res {
      Ok(tf) => Ok(tf),
      Err(err) => match err {
        TrackFileError::NotFound(_) => Self::create(path, flight_id),
        _ => Err(err),
      },
    }
  }

  pub fn create(path: PathBuf, flight_id: &str) -> Result<Self, TrackFileError> {
    let mut file = OpenOptions::new()
      .create(true)
      .write(true)
      .read(true)
      .open(&path)?;
    let header = Header::new(flight_id)?;
    let raw_header = to_raw(&header);
    file.write_all(&raw_header)?;
    Ok(Self {
      flight_id: flight_id.to_owned(),
      file,
      path,
    })
  }

  pub fn open(path: PathBuf, flight_id: &str) -> Result<Self, TrackFileError> {
    let res = OpenOptions::new().write(true).read(true).open(&path);
    match res {
      Ok(file) => {
        let tf = Self {
          flight_id: flight_id.to_owned(),
          file,
          path,
        };
        tf.check()?;
        Ok(tf)
      }
      Err(err) => match err.kind() {
        std::io::ErrorKind::NotFound => {
          let path = path.to_string_lossy().to_string();
          Err(TrackFileError::NotFound(path))
        }
        _ => Err(err.into()),
      },
    }
  }

  fn check(&self) -> Result<(), TrackFileError> {
    let header = self.read_file_header()?;
    if !header.check_magic() {
      Err(TrackFileError::InvalidMagicNumber)
    } else {
      let meta = std::fs::metadata(&self.path)?;
      let expected_len = (header.count() as usize) * Self::entry_size() + Self::header_size();
      let real_len = meta.len() as usize;
      if real_len != expected_len {
        Err(TrackFileError::InvalidFileLength(expected_len, real_len))
      } else {
        Ok(())
      }
    }
  }

  fn make_entry_buf() -> Vec<u8> {
    let mut buf = vec![];
    buf.resize(Self::entry_size(), 0);
    buf
  }

  fn make_header_buf() -> Vec<u8> {
    let mut buf = vec![];
    buf.resize(Self::header_size(), 0);
    buf
  }

  const fn entry_size() -> usize {
    size_of::<TrackFileEntry>()
  }

  const fn header_size() -> usize {
    size_of::<Header>()
  }

  fn read_file_header(&self) -> Result<Header, TrackFileError> {
    let mut buf = Self::make_header_buf();
    self.file.read_at(&mut buf, 0)?;
    from_raw(&buf, "header")
  }

  fn write_file_header(&mut self, header: &Header) -> Result<(), TrackFileError> {
    let buf = to_raw(header);
    self.file.write_at(&buf, 0)?;
    Ok(())
  }

  fn inc(&mut self) -> Result<(), TrackFileError> {
    let mut header = self.read_file_header()?;
    header.inc();
    self.write_file_header(&header)?;
    Ok(())
  }

  pub fn flight_id(&self) -> &str {
    &self.flight_id
  }

  pub fn mtime(&self) -> Result<DateTime<Utc>, TrackFileError> {
    let header = self.read_file_header()?;
    let secs = header.timestamp() / 1000;
    let nsecs = (header.timestamp() % 1000) * 1000;
    let dt = DateTime::from_timestamp(secs as i64, nsecs as u32).unwrap_or(Utc::now());
    Ok(dt)
  }

  pub fn count(&self) -> Result<u64, TrackFileError> {
    let header = self.read_file_header()?;
    Ok(header.count())
  }

  pub fn destroy(self) -> Result<(), TrackFileError> {
    std::fs::remove_file(self.path)?;
    Ok(())
  }

  pub fn set_departure(&mut self, departure: &str) -> Result<(), TrackFileError> {
    let mut header = self.read_file_header()?;
    header.set_departure(departure);
    self.write_file_header(&header)
  }

  pub fn set_arrival(&mut self, arrival: &str) -> Result<(), TrackFileError> {
    let mut header = self.read_file_header()?;
    header.set_arrival(arrival);
    self.write_file_header(&header)
  }

  pub fn get_departure(&self) -> Result<String, TrackFileError> {
    let header = self.read_file_header()?;
    Ok(header.get_departure())
  }

  pub fn get_arrival(&self) -> Result<String, TrackFileError> {
    let header = self.read_file_header()?;
    Ok(header.get_arrival())
  }

  pub fn append(&mut self, e: &TrackFileEntry) -> Result<(), TrackFileError> {
    let header = self.read_file_header()?;
    let count = header.count() as usize;

    let offset = if count < 2 {
      // if less than 2 points exist, append only
      0
    } else {
      let mut last_two = self.read_multiple_at(count - 2, 2)?;
      let last = last_two.pop().unwrap();
      let prev = last_two.pop().unwrap();
      if last == prev && prev == *e {
        // if the last two points are equal and the new one equals to them
        // replace the last one, overwriting only timestamp
        -(Self::entry_size() as i64)
      } else {
        // otherwise, append
        0
      }
    };

    if offset == 0 {
      self.inc()?
    }

    let data = to_raw(e);
    self.file.seek(SeekFrom::End(offset))?;
    self.file.write_all(&data)?;
    Ok(())
  }

  pub fn read_at(&self, pos: usize) -> Result<TrackFileEntry, TrackFileError> {
    let header = self.read_file_header()?;
    if pos as u64 >= header.count() {
      Err(TrackFileError::IndexError(pos))
    } else {
      let mut buf = Self::make_entry_buf();
      let offset = Self::header_size() + pos * Self::entry_size();
      self.file.read_at(&mut buf, offset as u64)?;
      let e = from_raw(&buf, "track entry")?;
      Ok(e)
    }
  }

  pub fn read_multiple_at(
    &self,
    pos: usize,
    len: usize,
  ) -> Result<Vec<TrackFileEntry>, TrackFileError> {
    let header = self.read_file_header()?;
    let count = header.count() as usize;
    let mut len = len;

    if pos + len > count {
      len = count - pos;
    }

    if len < 1 {
      return Ok(Vec::new());
    }

    let mut buf = vec![];
    let entry_len = Self::entry_size();
    buf.resize(len * entry_len, 0);

    let offset = Self::header_size() + pos * entry_len;
    self.file.read_at(&mut buf, offset as u64)?;

    let mut entries = vec![];
    for idx in 0..len {
      let start = idx * entry_len;
      let end = (idx + 1) * entry_len;
      let e = from_raw(&buf[start..end], "track entry")?;
      entries.push(e);
    }

    Ok(entries)
  }

  pub fn read_all(&self) -> Result<Vec<TrackFileEntry>, TrackFileError> {
    let header = self.read_file_header()?;

    let mut buf = Self::make_entry_buf();
    let mut res = vec![];
    for idx in 0..header.count() {
      let idx = idx as usize;
      let offset = Self::header_size() + idx * Self::entry_size();
      self.file.read_at(&mut buf, offset as u64)?;
      let tp = from_raw(&buf, "track entry")?;
      res.push(tp);
    }
    Ok(res)
  }

  pub fn get_header(&self) -> Result<Header, TrackFileError> {
    self.read_file_header()
  }
}
