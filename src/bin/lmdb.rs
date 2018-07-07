#![feature(test)]

extern crate lmdb;
extern crate tempdir;

use lmdb::{
    Environment,
    Error,
    Transaction,
    WriteFlags,
};
use tempdir::TempDir;

fn main() {
    // Based on test_put_get_del() in https://github.com/danburkert/lmdb-rs.

    let dir = TempDir::new("test").unwrap();
    let env = Environment::new().open(dir.path()).unwrap();
    let db = env.open_db(None).unwrap();

    let mut txn = env.begin_rw_txn().unwrap();
    txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
    txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
    txn.put(db, b"key3", b"val3", WriteFlags::empty()).unwrap();
    txn.commit().unwrap();

    let mut txn = env.begin_rw_txn().unwrap();
    assert_eq!(b"val1", txn.get(db, b"key1").unwrap());
    assert_eq!(b"val2", txn.get(db, b"key2").unwrap());
    assert_eq!(b"val3", txn.get(db, b"key3").unwrap());
    assert_eq!(txn.get(db, b"key"), Err(Error::NotFound));

    txn.del(db, b"key1", None).unwrap();
    assert_eq!(txn.get(db, b"key1"), Err(Error::NotFound));
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;
    extern crate walkdir;

    use lmdb::{
        Cursor,
        Environment,
        EnvironmentFlags,
        Transaction,
        WriteFlags,
    };
    use self::rand::{Rng, thread_rng};
    use self::test::{Bencher, black_box};
    use self::walkdir::WalkDir;
    use std::{thread, time};
    use tempdir::TempDir;

    fn get_key(n: u32) -> [u8; 4] {
        n.to_bytes()
    }

    fn get_value(n: u32) -> Vec<u8> {
        format!("data{}", n).into_bytes()
    }

    fn get_pair(n: u32) -> ([u8; 4], Vec<u8>) {
        (get_key(n), get_value(n))
    }

    fn setup_bench_db(num_pairs: u32) -> (TempDir, Environment) {
        let dir = TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path()).unwrap();

        {
            let db = env.open_db(None).unwrap();
            let mut txn = env.begin_rw_txn().unwrap();
            for i in 0..num_pairs {
                txn.put(db, &get_key(i), &get_value(i), WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        }
        (dir, env)
    }

    #[bench]
    fn bench_open_db(b: &mut Bencher) {
        let dir = TempDir::new("bench_open_db").unwrap();
        let path = dir.path();

        // Create the database first so we only measure the time to open
        // an existing database.
        {
            let env = Environment::new().open(path).unwrap();
            let _db = env.open_db(None).unwrap();
        }

        b.iter(|| {
            let env = Environment::new().open(path).unwrap();
            let _db = env.open_db(None).unwrap();
        });
    }

    #[bench]
    fn bench_put_seq_sync(b: &mut Bencher) {
        let num_pairs = 100;
        let dir = TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path()).unwrap();
        let db = env.open_db(None).unwrap();

        let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();

        b.iter(|| {
            let mut txn = env.begin_rw_txn().unwrap();
            for (key, value) in &pairs {
                txn.put(db, key, value, WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        });
    }

    #[bench]
    fn bench_put_seq_async(b: &mut Bencher) {
        let num_pairs = 100;
        let dir = TempDir::new("test").unwrap();
        // LMDB writes are sync by default.  Set the MAP_ASYNC and WRITE_MAP
        // environment flags to make them async (along with using a writeable
        // memory map).
        let env = Environment::new()
            .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
            .open(dir.path()).unwrap();
        let db = env.open_db(None).unwrap();

        let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();

        b.iter(|| {
            let mut txn = env.begin_rw_txn().unwrap();
            for (key, value) in &pairs {
                txn.put(db, key, value, WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        });
    }

    #[bench]
    fn bench_put_rand_sync(b: &mut Bencher) {
        let num_pairs = 100;
        let dir = TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path()).unwrap();
        let db = env.open_db(None).unwrap();

        let mut pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
        thread_rng().shuffle(&mut pairs[..]);

        b.iter(|| {
            let mut txn = env.begin_rw_txn().unwrap();
            for (key, value) in &pairs {
                txn.put(db, key, value, WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        });
    }

    #[bench]
    fn bench_put_rand_async(b: &mut Bencher) {
        let num_pairs = 100;
        let dir = TempDir::new("test").unwrap();
        // LMDB writes are sync by default.  Set the MAP_ASYNC and WRITE_MAP
        // environment flags to make them async (along with using a writeable
        // memory map).
        let env = Environment::new()
            .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
            .open(dir.path()).unwrap();
        let db = env.open_db(None).unwrap();

        let mut pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
        thread_rng().shuffle(&mut pairs[..]);

        b.iter(|| {
            let mut txn = env.begin_rw_txn().unwrap();
            for (key, value) in &pairs {
                txn.put(db, key, value, WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        });
    }

    #[bench]
    fn bench_get_seq(b: &mut Bencher) {
        let num_pairs = 100;
        let (_dir, env) = setup_bench_db(num_pairs);
        let db = env.open_db(None).unwrap();
        let txn = env.begin_ro_txn().unwrap();

        let keys: Vec<[u8; 4]> = (0..num_pairs).map(|n| get_key(n)).collect();

        b.iter(|| {
            let mut i = 0usize;
            for key in &keys {
                i = i + txn.get(db, key).unwrap().len();
            }
            black_box(i);
        });
    }

    #[bench]
    fn bench_get_rand(b: &mut Bencher) {
        let num_pairs = 100;
        let (_dir, env) = setup_bench_db(num_pairs);
        let db = env.open_db(None).unwrap();
        let txn = env.begin_ro_txn().unwrap();

        let mut keys: Vec<[u8; 4]> = (0..num_pairs).map(|n| get_key(n)).collect();
        thread_rng().shuffle(&mut keys[..]);

        b.iter(|| {
            let mut i = 0usize;
            for key in &keys {
                i = i + txn.get(db, key).unwrap().len();
            }
            black_box(i);
        });
    }

    /// Benchmark of iterator sequential read performance.
    #[bench]
    fn bench_get_seq_iter(b: &mut Bencher) {
        let num_pairs = 100;
        let (_dir, env) = setup_bench_db(num_pairs);
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

            black_box(i);
            assert_eq!(count, num_pairs);
        });
    }

    // This measures space on disk, not time, reflecting the space taken
    // by a database on disk into the time it takes the benchmark to complete.
    #[bench]
    fn bench_db_size(b: &mut Bencher) {
        let num_pairs = 100;
        let (dir, _env) = setup_bench_db(num_pairs);
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
