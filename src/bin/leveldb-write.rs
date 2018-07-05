#![feature(test)]

extern crate rand;
extern crate tempdir;
extern crate leveldb;
extern crate test;

use tempdir::TempDir;
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::kv::KV;
use leveldb::options::{Options,WriteOptions,ReadOptions};

fn main() {
  let tempdir = TempDir::new("demo").unwrap();
  let path = tempdir.path();

  let mut options = Options::new();
  options.create_if_missing = true;
  let database = match Database::open(path, options) {
      Ok(db) => { db },
      Err(e) => { panic!("failed to open database: {:?}", e) }
  };

  let write_opts = WriteOptions::new();
  match database.put(write_opts, 1, &[1]) {
      Ok(_) => { () },
      Err(e) => { panic!("failed to write to database: {:?}", e) }
  };

  let read_opts = ReadOptions::new();
  let res = database.get(read_opts, 1);

  match res {
    Ok(data) => {
      assert!(data.is_some());
      assert_eq!(data, Some(vec![1]));
    }
    Err(e) => { panic!("failed reading data: {:?}", e) }
  }

  let read_opts = ReadOptions::new();
  let mut iter = database.iter(read_opts);
  let entry = iter.next();
  assert_eq!(
    entry,
    Some((1, vec![1]))
  );
}

#[cfg(test)]
mod tests {
    use leveldb::database::Database;
    use leveldb::kv::KV;
    use leveldb::options::{Options, ReadOptions, WriteOptions};
    use rand::{Rng, XorShiftRng};
    use tempdir::TempDir;
    use test::{Bencher, black_box};

    pub fn get_key(n: u32) -> String {
        format!("key{}", n)
    }

    pub fn get_data(n: u32) -> String {
        format!("data{}", n)
    }

    pub fn setup_bench_db<'a>(num_rows: u32) -> TempDir {
        let tempdir = TempDir::new("demo").unwrap();
        let path_buf = tempdir.path().to_path_buf();
        let path = path_buf.as_path();

        let mut options = Options::new();
        options.create_if_missing = true;
        let database = match Database::open(path, options) {
            Ok(db) => { db },
            Err(e) => { panic!("failed to open database: {:?}", e) }
        };

        let write_opts = WriteOptions::new();

        for i in 0..num_rows {
            match database.put(write_opts, i as i32, get_data(i).as_str().as_bytes()) {
                Ok(_) => { () },
                Err(e) => { panic!("failed to write to database: {:?}", e) }
            };
        }

        tempdir
    }

    #[bench]
    fn bench_setup_bench_db(b: &mut Bencher) {
        let n = 100u32;
        b.iter(|| {
            let _dir = setup_bench_db(n);
        });
    }

    #[bench]
    fn bench_get_rand(b: &mut Bencher) {
        let n = 100u32;
        let tempdir = setup_bench_db(n);
        let path = tempdir.path();

        let options = Options::new();
        let database: Database<i32> = match Database::open(path, options) {
            Ok(db) => { db },
            Err(e) => { panic!("failed to open database: {:?}", e) }
        };

        let mut keys: Vec<i32> = (0..n as i32).collect();
        XorShiftRng::new_unseeded().shuffle(&mut keys[..]);

        b.iter(|| {
            let mut i = 0usize;
            for key in &keys {
                let read_opts = ReadOptions::new();
                i = i + database.get(read_opts, key).unwrap().unwrap().len();
            }
            black_box(i);
        });
    }

}
