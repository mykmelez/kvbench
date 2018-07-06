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

    pub fn get_key(n: u32) -> i32 {
        n as i32
    }

    pub fn get_value(n: u32) -> Vec<u8> {
        format!("data{}", n).into_bytes()
    }

    pub fn get_pair(n: u32) -> (i32, Vec<u8>) {
        (get_key(n), get_value(n))
    }

    pub fn setup_bench_db(num_pairs: u32) -> TempDir {
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
        for i in 0..num_pairs {
            batch.put(i as i32, &get_value(i));
        }
        let write_opts = WriteOptions::new();
        match database.write(write_opts, batch) {
            Ok(_) => { () },
            Err(e) => { panic!("failed to write to database: {:?}", e) }
        };

        tempdir
    }

    #[bench]
    fn bench_open_db(b: &mut Bencher) {
        let dir = TempDir::new("bench_open_db").unwrap();
        let path = dir.path();

        // Create the database first so we only measure the time to open
        // an existing database.
        {
            let mut options = Options::new();
            options.create_if_missing = true;
            let _db: Database<i32> = Database::open(path, options).unwrap();
        }

        b.iter(|| {
            let _db: Database<i32> = Database::open(path, Options::new()).unwrap();
        });
    }

    #[bench]
    fn bench_put_seq(b: &mut Bencher) {
        let dir = TempDir::new("bench_put_seq").unwrap();
        let path = dir.path();
        let num_pairs = 100;

        let mut options = Options::new();
        options.create_if_missing = true;
        let db: Database<i32> = Database::open(path, options).unwrap();

        let pairs: Vec<(i32, Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();

        b.iter(|| {
            let batch = &mut Writebatch::new();
            for (key, value) in &pairs {
                batch.put(*key, value);
            }
            let write_opts = WriteOptions::new();
            db.write(write_opts, batch).unwrap();
        });
    }

    #[bench]
    fn bench_put_rand(b: &mut Bencher) {
        let dir = TempDir::new("bench_put_rand").unwrap();
        let path = dir.path();
        let num_pairs = 100;

        let mut options = Options::new();
        options.create_if_missing = true;
        let db: Database<i32> = Database::open(path, options).unwrap();

        let mut pairs: Vec<(i32, Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
        thread_rng().shuffle(&mut pairs[..]);

        b.iter(|| {
            let batch = &mut Writebatch::new();
            for (key, value) in &pairs {
                batch.put(*key, value);
            }
            let write_opts = WriteOptions::new();
            db.write(write_opts, batch).unwrap();
        });
    }

    #[bench]
    fn bench_get_seq(b: &mut Bencher) {
        let num_pairs = 100;
        let tempdir = setup_bench_db(num_pairs);
        let path = tempdir.path();

        let options = Options::new();
        let database: Database<i32> = Database::open(path, options).unwrap();

        let keys: Vec<i32> = (0..num_pairs as i32).collect();

        b.iter(|| {
            let mut i = 0usize;
            for key in &keys {
                let read_opts = ReadOptions::new();
                i = i + database.get(read_opts, key).unwrap().unwrap().len();
            }
            black_box(i);
        });
    }

    #[bench]
    fn bench_get_rand(b: &mut Bencher) {
        let num_pairs = 100;
        let tempdir = setup_bench_db(num_pairs);
        let path = tempdir.path();

        let options = Options::new();
        let database: Database<i32> = Database::open(path, options).unwrap();

        let mut keys: Vec<i32> = (0..num_pairs as i32).collect();
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
