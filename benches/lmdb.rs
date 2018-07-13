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

#[macro_use]
extern crate lazy_static;

extern crate libc;
extern crate lmdb;
extern crate page_size;
extern crate rand;
extern crate tempdir;
extern crate walkdir;

use criterion::Criterion;

use libc::size_t;

use lmdb::{
    Cursor,
    Database,
    Environment,
    EnvironmentBuilder,
    EnvironmentFlags,
    Transaction,
    WriteFlags,
};

use rand::{
    random,
    thread_rng,
    Rng,
};

use std::{
    thread,
    time,
};

use tempdir::TempDir;
use walkdir::WalkDir;

// To accommodate benchmarking datastores with many pairs and large values,
// we increase the size of the map to fit the largest data sets we bench.
//
// Note that mdb_set_map_size
// <http://www.lmdb.tech/doc/group__mdb.html#gaa2506ec8dab3d969b0e609cd82e619e5>
// claims the default map size is 10MiB, while DEFAULT_MAPSIZE
// <http://www.lmdb.tech/doc/group__internal.html#ga506f893519db205966f7988c03c920f5>
// claims it's 1MiB.  The latter seems correct in my testing, since benches
// that fail at the default size succeed when it's manually set to 10MiB.
//
const MB: size_t = 1024 * 1024;
const MAP_SIZE: size_t = 5 * MB;

// We parameterize benchmarks across both the number of KV pairs we write to
// (or read from) a datastore and the sizes of the values we write (or read).
//
// It might make sense to also parameterize across the sizes of the keys,
// for which we'd need to define a struct that implements the db_key::Key trait,
// since the leveldb crate delegates to the db_key crate to define key types,
// and the only implementation of Key in the db_key crate itself is for u32,
// which always has the same size: four bytes.
//
// (The lmdb crate defines key types as any type that implements AsRef<[u8]>,
// so we don't need to do anything special to parameterize across key sizes
// for that storage engine.)
//
const PAIR_COUNTS: [u32; 3] = [1, 100, 1000];
const VALUE_SIZES: [usize; 3] = [1, 100, 1000];

lazy_static! {
    // A collection of tuples (num_pairs, size_values) representing every
    // combination of numbers of pairs and sizes of values, which we use
    // to benchmark storage engine performance across various shapes of data.
    static ref PARAMS: Vec<(u32, usize)> =
        PAIR_COUNTS.iter().flat_map(|&m| VALUE_SIZES.iter().map(move |&n| (m, n))).collect();
}

fn get_key(n: u32) -> [u8; 4] {
    let b1: u8 = ((n >> 24) & 0xff) as u8;
    let b2: u8 = ((n >> 16) & 0xff) as u8;
    let b3: u8 = ((n >> 8) & 0xff) as u8;
    let b4: u8 = (n & 0xff) as u8;
    [b1, b2, b3, b4]
}

fn get_value(size_values: usize) -> Vec<u8> {
    (0..size_values).map(|_| random()).collect()
}

fn get_pair(num_pairs: u32, size_values: usize) -> ([u8; 4], Vec<u8>) {
    (get_key(num_pairs), get_value(size_values))
}

fn get_env() -> EnvironmentBuilder {
    // The map size should be a multiple of the system page size.
    assert_eq!(MAP_SIZE % page_size::get(), 0);

    *Environment::new().set_map_size(MAP_SIZE)
}

fn setup_bench_db(num_pairs: u32, size_values: usize) -> (TempDir, Environment) {
    let dir = TempDir::new("test").unwrap();
    let env = get_env().open(dir.path()).unwrap();

    {
        let db = env.open_db(None).unwrap();
        let mut txn = env.begin_rw_txn().unwrap();
        for i in 0..num_pairs {
            txn.put(db, &get_key(i), &get_value(size_values), WriteFlags::empty()).unwrap();
        }
        txn.commit().unwrap();
    }

    (dir, env)
}

fn bench_open_db(c: &mut Criterion) {
    let dir = TempDir::new("bench_open_db").unwrap();

    // Create the database first so we only measure the time to open
    // an existing database.
    {
        let env = Environment::new().open(dir.path()).unwrap();
        let _db = env.open_db(None).unwrap();
    }

    c.bench_function("lmdb_open_db", move |b| {
        b.iter(|| {
            let env = Environment::new().open(dir.path()).unwrap();
            let _db = env.open_db(None).unwrap();
        })
    });
}

fn lmdb_put(env: &Environment, db: Database, pairs: &Vec<([u8; 4], Vec<u8>)>) {
    let mut txn = env.begin_rw_txn().unwrap();
    for (key, value) in pairs {
        txn.put(db, key, value, WriteFlags::empty()).unwrap();
    }
    txn.commit().unwrap();
}

fn bench_put_seq_sync(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_put_seq_sync",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let dir = TempDir::new("test").unwrap();
            let env = get_env().open(dir.path()).unwrap();
            let db = env.open_db(None).unwrap();
            let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n, size_values)).collect();

            b.iter(|| lmdb_put(&env, db, &pairs))
        },
        PARAMS.iter(),
    );
}

fn bench_put_seq_async(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_put_seq_async",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let dir = TempDir::new("test").unwrap();
            // LMDB writes are sync by default.  Set the MAP_ASYNC and WRITE_MAP
            // environment flags to make them async (along with using a writeable
            // memory map).
            let env = get_env()
                .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
                .open(dir.path())
                .unwrap();
            let db = env.open_db(None).unwrap();
            let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n, size_values)).collect();

            b.iter(|| lmdb_put(&env, db, &pairs))
        },
        PARAMS.iter(),
    );
}

fn bench_put_rand_sync(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_put_rand_sync",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let dir = TempDir::new("test").unwrap();
            let env = get_env().open(dir.path()).unwrap();
            let db = env.open_db(None).unwrap();
            let mut pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n, size_values)).collect();
            thread_rng().shuffle(&mut pairs[..]);

            b.iter(|| lmdb_put(&env, db, &pairs))
        },
        PARAMS.iter(),
    );
}

fn bench_put_rand_async(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_put_rand_async",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let dir = TempDir::new("test").unwrap();
            // LMDB writes are sync by default.  Set the MAP_ASYNC and WRITE_MAP
            // environment flags to make them async (along with using a writeable
            // memory map).
            let env = get_env()
                .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
                .open(dir.path())
                .unwrap();
            let db = env.open_db(None).unwrap();
            let mut pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n, size_values)).collect();
            thread_rng().shuffle(&mut pairs[..]);

            b.iter(|| lmdb_put(&env, db, &pairs))
        },
        PARAMS.iter(),
    );
}

fn bench_get_seq(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_get_seq",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let (_dir, env) = setup_bench_db(num_pairs, size_values);
            let db = env.open_db(None).unwrap();
            let keys: Vec<[u8; 4]> = (0..num_pairs).map(|n| get_key(n)).collect();

            b.iter(|| {
                let txn = env.begin_ro_txn().unwrap();
                let mut i = 0usize;
                for key in &keys {
                    i = i + txn.get(db, key).unwrap().len();
                }
            })
        },
        PARAMS.iter(),
    );
}

fn bench_get_rand(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_get_rand",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let (_dir, env) = setup_bench_db(num_pairs, size_values);
            let db = env.open_db(None).unwrap();
            let mut keys: Vec<[u8; 4]> = (0..num_pairs).map(|n| get_key(n)).collect();
            thread_rng().shuffle(&mut keys[..]);

            let txn = env.begin_ro_txn().unwrap();
            b.iter(|| {
                let mut i = 0usize;
                for key in &keys {
                    i = i + txn.get(db, key).unwrap().len();
                }
            })
        },
        PARAMS.iter(),
    );
}

/// Benchmark of iterator sequential read performance.
fn bench_get_seq_iter(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_get_seq_iter",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let (_dir, env) = setup_bench_db(num_pairs, size_values);
            let db = env.open_db(None).unwrap();
            let txn = env.begin_ro_txn().unwrap();

            b.iter(|| {
                let mut cursor = txn.open_ro_cursor(db).unwrap();
                let mut i = 0;
                let mut count = 0u32;
                for (key, data) in cursor.iter() {
                    i = i + key.len() + data.len();
                    count = count + 1;
                }
                assert_eq!(count, num_pairs);
            })
        },
        PARAMS.iter(),
    );
}

// This measures space on disk, not time, reflecting the space taken
// by a database on disk into the time it takes the benchmark to complete.
// It is non-obvious to me that this is an accurate way to measure space,
// much less an optimal one.
fn bench_db_size(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_db_size",
        |b, &&t| {
            let (num_pairs, size_values) = t;
            let (dir, _env) = setup_bench_db(num_pairs, size_values);
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
            })
        },
        PARAMS.iter(),
    );
}

criterion_group!(
    benches,
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
