use trackdump::dump_trackfile;

fn main() {
  let res = dump_trackfile();
  if let Err(err) = res {
    println!("Error dumping track file: {}", err);
  }
}
