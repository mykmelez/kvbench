// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Portions of this code were copied or adapted from lmdb-rs
// <https://github.com/danburkert/lmdb-rs>, which is written by Dan Burkert.

#![feature(test)]

extern crate db_key;
extern crate leveldb;
extern crate tempdir;

use db_key::Key;
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
use tempdir::TempDir;

fn main() {
    let dir = TempDir::new("example").unwrap();
    let mut options = Options::new();
    options.create_if_missing = true;
    let database: Database<i32> = Database::open(dir.path(), options).unwrap();

    let key1 = Key::from_u8(b"key1");
    let key2 = Key::from_u8(b"key2");
    let key3 = Key::from_u8(b"key3");

    let batch = &mut Writebatch::new();
    batch.put(key1, b"val1");
    batch.put(key2, b"val2");
    batch.put(key3, b"val3");
    database.write(WriteOptions::new(), batch).unwrap();

    assert_eq!(database.get(ReadOptions::new(), key1).unwrap().unwrap(), b"val1");
    assert_eq!(database.get(ReadOptions::new(), key2).unwrap().unwrap(), b"val2");
    assert_eq!(database.get(ReadOptions::new(), key3).unwrap().unwrap(), b"val3");

    database.delete(WriteOptions::new(), key1).unwrap();
    assert_eq!(database.get(ReadOptions::new(), key1).unwrap(), None);
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;
    extern crate walkdir;

    use self::rand::{
        thread_rng,
        Rng,
    };
    use self::test::{
        black_box,
        Bencher,
    };
    use self::walkdir::WalkDir;
    use leveldb::database::batch::{
        Batch,
        Writebatch,
    };
    use leveldb::database::Database;
    use leveldb::iterator::Iterable;
    use leveldb::kv::KV;
    use leveldb::options::{
        Options,
        ReadOptions,
        WriteOptions,
    };
    use std::{
        thread,
        time,
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
        let dir = TempDir::new("demo").unwrap();

        let mut options = Options::new();
        options.create_if_missing = true;
        let database = Database::open(dir.path(), options).unwrap();

        let batch = &mut Writebatch::new();
        for i in 0..num_pairs {
            batch.put(i as i32, &get_value(i));
        }
        let mut write_opts = WriteOptions::new();
        write_opts.sync = true;
        database.write(write_opts, batch).unwrap();

        dir
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
    fn bench_put_seq_sync(b: &mut Bencher) {
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
            let mut write_opts = WriteOptions::new();
            // LevelDB writes are async by default.  Set the WriteOptions::sync
            // flag to true to make them sync.
            write_opts.sync = true;
            db.write(write_opts, batch).unwrap();
        });
    }

    #[bench]
    fn bench_put_seq_async(b: &mut Bencher) {
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
    fn bench_put_rand_sync(b: &mut Bencher) {
        let dir = TempDir::new("bench_put_rand_sync").unwrap();
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
            let mut write_opts = WriteOptions::new();
            // LevelDB writes are async by default.  Set the WriteOptions::sync
            // flag to true to make them sync.
            write_opts.sync = true;
            db.write(write_opts, batch).unwrap();
        });
    }

    #[bench]
    fn bench_put_rand_async(b: &mut Bencher) {
        let dir = TempDir::new("bench_put_rand_async").unwrap();
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
        let dir = setup_bench_db(num_pairs);
        let path = dir.path();

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
        let dir = setup_bench_db(num_pairs);
        let path = dir.path();

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

    #[bench]
    fn bench_get_seq_iter(b: &mut Bencher) {
        let num_pairs = 100;
        let dir = setup_bench_db(num_pairs);
        let path = dir.path();

        let options = Options::new();
        let database: Database<i32> = Database::open(path, options).unwrap();

        let mut keys: Vec<i32> = (0..num_pairs as i32).collect();
        thread_rng().shuffle(&mut keys[..]);

        b.iter(|| {
            let mut i = 0;
            let mut count = 0u32;
            let read_opts = ReadOptions::new();

            for (key, data) in database.iter(read_opts) {
                i = i + key as usize + data.len();
                count = count + 1;
            }

            black_box(i);
            assert_eq!(count, num_pairs);
        });
    }

    // This measures space on disk, not time, reflecting the space taken
    // by a database on disk into the time it takes the benchmark to complete.
    #[bench]
    fn bench_db_size(b: &mut Bencher) {
        let num_pairs = 100;
        let dir = setup_bench_db(num_pairs);
        let mut total_size = 0;

        for entry in WalkDir::new(dir.path()) {
            let metadata = entry.unwrap().metadata().unwrap();
            if metadata.is_file() {
                total_size += metadata.len();
            }
        }

        b.iter(|| {
            // Convert size on disk to benchmark time by sleeping
            // for the total_size number of nanoseconds.
            thread::sleep(time::Duration::from_nanos(total_size));
        });
    }
}
