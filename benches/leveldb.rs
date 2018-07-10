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

#[macro_use]
extern crate criterion;
use criterion::Criterion;

extern crate db_key;
extern crate leveldb;
extern crate rand;
extern crate tempdir;
extern crate walkdir;

use self::rand::{
    thread_rng,
    Rng,
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

fn bench_open_db(c: &mut Criterion) {
    let dir = TempDir::new("bench_open_db").unwrap();

    // Create the database first so we only measure the time to open
    // an existing database.
    {
        let mut options = Options::new();
        options.create_if_missing = true;
        let _db: Database<i32> = Database::open(dir.path(), options).unwrap();
    }

    c.bench_function("open_db", move |b| b.iter(|| {
        let _db: Database<i32> = Database::open(dir.path(), Options::new()).unwrap();
    }));
}

fn bench_put_seq_sync(c: &mut Criterion) {
    let dir = TempDir::new("bench_put_seq").unwrap();
    let path = dir.path();
    let num_pairs = 100;

    let mut options = Options::new();
    options.create_if_missing = true;
    let db: Database<i32> = Database::open(path, options).unwrap();

    let pairs: Vec<(i32, Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();

    c.bench_function("put_seq_sync", move |b| b.iter(|| {
        let batch = &mut Writebatch::new();
        for (key, value) in &pairs {
            batch.put(*key, value);
        }
        let mut write_opts = WriteOptions::new();
        // LevelDB writes are async by default.  Set the WriteOptions::sync
        // flag to true to make them sync.
        write_opts.sync = true;
        db.write(write_opts, batch).unwrap();
    }));
}

fn bench_put_seq_async(c: &mut Criterion) {
    let dir = TempDir::new("bench_put_seq").unwrap();
    let path = dir.path();
    let num_pairs = 100;

    let mut options = Options::new();
    options.create_if_missing = true;
    let db: Database<i32> = Database::open(path, options).unwrap();

    let pairs: Vec<(i32, Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();

    c.bench_function("put_seq_async", move |b| b.iter(|| {
        let batch = &mut Writebatch::new();
        for (key, value) in &pairs {
            batch.put(*key, value);
        }
        let write_opts = WriteOptions::new();
        db.write(write_opts, batch).unwrap();
    }));
}

fn bench_put_rand_sync(c: &mut Criterion) {
    let dir = TempDir::new("bench_put_rand_sync").unwrap();
    let path = dir.path();
    let num_pairs = 100;

    let mut options = Options::new();
    options.create_if_missing = true;
    let db: Database<i32> = Database::open(path, options).unwrap();

    let mut pairs: Vec<(i32, Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
    thread_rng().shuffle(&mut pairs[..]);

    c.bench_function("put_rand_sync", move |b| b.iter(|| {
        let batch = &mut Writebatch::new();
        for (key, value) in &pairs {
            batch.put(*key, value);
        }
        let mut write_opts = WriteOptions::new();
        // LevelDB writes are async by default.  Set the WriteOptions::sync
        // flag to true to make them sync.
        write_opts.sync = true;
        db.write(write_opts, batch).unwrap();
    }));
}

fn bench_put_rand_async(c: &mut Criterion) {
    let dir = TempDir::new("bench_put_rand_async").unwrap();
    let path = dir.path();
    let num_pairs = 100;

    let mut options = Options::new();
    options.create_if_missing = true;
    let db: Database<i32> = Database::open(path, options).unwrap();

    let mut pairs: Vec<(i32, Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
    thread_rng().shuffle(&mut pairs[..]);

    c.bench_function("put_rand_async", move |b| b.iter(|| {
        let batch = &mut Writebatch::new();
        for (key, value) in &pairs {
            batch.put(*key, value);
        }
        let write_opts = WriteOptions::new();
        db.write(write_opts, batch).unwrap();
    }));
}

fn bench_get_seq(c: &mut Criterion) {
    let num_pairs = 100;
    let dir = setup_bench_db(num_pairs);
    let path = dir.path();

    let options = Options::new();
    let database: Database<i32> = Database::open(path, options).unwrap();

    let keys: Vec<i32> = (0..num_pairs as i32).collect();

    c.bench_function("get_seq", move |b| b.iter(|| {
        let mut i = 0usize;
        for key in &keys {
            let read_opts = ReadOptions::new();
            i = i + database.get(read_opts, key).unwrap().unwrap().len();
        }
    }));
}

fn bench_get_rand(c: &mut Criterion) {
    let num_pairs = 100;
    let dir = setup_bench_db(num_pairs);
    let path = dir.path();

    let options = Options::new();
    let database: Database<i32> = Database::open(path, options).unwrap();

    let mut keys: Vec<i32> = (0..num_pairs as i32).collect();
    thread_rng().shuffle(&mut keys[..]);

    c.bench_function("get_rand", move |b| b.iter(|| {
        let mut i = 0usize;
        for key in &keys {
            let read_opts = ReadOptions::new();
            i = i + database.get(read_opts, key).unwrap().unwrap().len();
        }
    }));
}

fn bench_get_seq_iter(c: &mut Criterion) {
    let num_pairs = 100;
    let dir = setup_bench_db(num_pairs);
    let path = dir.path();

    let options = Options::new();
    let database: Database<i32> = Database::open(path, options).unwrap();

    let mut keys: Vec<i32> = (0..num_pairs as i32).collect();
    thread_rng().shuffle(&mut keys[..]);

    c.bench_function("get_seq_iter", move |b| b.iter(|| {
        let mut i = 0;
        let mut count = 0u32;
        let read_opts = ReadOptions::new();

        for (key, data) in database.iter(read_opts) {
            i = i + key as usize + data.len();
            count = count + 1;
        }

        assert_eq!(count, num_pairs);
    }));
}

// This measures space on disk, not time, reflecting the space taken
// by a database on disk into the time it takes the benchmark to complete.
fn bench_db_size(c: &mut Criterion) {
    let num_pairs = 100;
    let dir = setup_bench_db(num_pairs);
    let mut total_size = 0;

    for entry in WalkDir::new(dir.path()) {
        let metadata = entry.unwrap().metadata().unwrap();
        if metadata.is_file() {
            total_size += metadata.len();
        }
    }

    c.bench_function("db_size", move |b| b.iter(|| {
        // Convert size on disk to benchmark time by sleeping
        // for the total_size number of nanoseconds.
        thread::sleep(time::Duration::from_nanos(total_size));
    }));
}

criterion_group!(benches,
    bench_open_db,
    bench_put_seq_sync,
    bench_put_seq_async,
    bench_put_rand_sync,
    bench_put_rand_async,
    bench_get_seq,
    bench_get_rand,
    bench_get_seq_iter,
    bench_db_size,
);
criterion_main!(benches);
