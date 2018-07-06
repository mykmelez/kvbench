#![feature(test)]

extern crate leveldb;
extern crate tempdir;

use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{
    Options,
    WriteOptions,
    ReadOptions,
};
use tempdir::TempDir;

fn main() {
    // Based on the example in the README for https://github.com/skade/leveldb.

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
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;

    use leveldb::database::batch::{
        Batch,
        Writebatch,
    };
    use leveldb::database::Database;
    use leveldb::kv::KV;
    use leveldb::options::{
        Options,
        ReadOptions,
        WriteOptions,
    };
    use self::rand::{
        Rng,
        thread_rng,
    };
    use self::test::{
        Bencher,
        black_box,
    };
    use tempdir::TempDir;

    pub fn get_value(n: u32) -> String {
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
            match database.put(write_opts, i as i32, get_value(i).as_str().as_bytes()) {
                Ok(_) => { () },
                Err(e) => { panic!("failed to write to database: {:?}", e) }
            };
        }

        tempdir
    }

    pub fn setup_bench_db_batch<'a>(num_rows: u32) -> TempDir {
        let tempdir = TempDir::new("demo").unwrap();
        let path_buf = tempdir.path().to_path_buf();
        let path = path_buf.as_path();

        let mut options = Options::new();
        options.create_if_missing = true;
        let database = match Database::open(path, options) {
            Ok(db) => { db },
            Err(e) => { panic!("failed to open database: {:?}", e) }
        };

        let batch = &mut Writebatch::new();
        for i in 0..num_rows {
            batch.put(i as i32, get_value(i).as_str().as_bytes());
        }
        let write_opts = WriteOptions::new();
        match database.write(write_opts, batch) {
            Ok(_) => { () },
            Err(e) => { panic!("failed to write to database: {:?}", e) }
        };

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
    fn bench_setup_bench_db_batch(b: &mut Bencher) {
        let n = 100u32;
        b.iter(|| {
            let _dir = setup_bench_db_batch(n);
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
        thread_rng().shuffle(&mut keys[..]);

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
