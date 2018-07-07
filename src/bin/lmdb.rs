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

    use lmdb::{
        Cursor,
        Environment,
        Transaction,
        WriteFlags,
    };
    use tempdir::TempDir;

    use self::rand::{Rng, thread_rng};
    use self::test::{Bencher, black_box};

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
    fn bench_put_seq(b: &mut Bencher) {
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
    fn bench_put_rand(b: &mut Bencher) {
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

    #[bench]
    fn bench_delete(b: &mut Bencher) {
        let num_pairs = 100;
        let (_dir, env) = setup_bench_db(num_pairs);
        let db = env.open_db(None).unwrap();

        let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();

        b.iter(|| {
            let mut txn = env.begin_rw_txn().unwrap();
            for (key, value) in &pairs {
                txn.put(db, key, value, WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
            let mut txn = env.begin_rw_txn().unwrap();
            for (key, _value) in &pairs {
                txn.del(db, key, None).unwrap();
            }
        });
    }

    /// Benchmark of iterator sequential read performance.
    #[bench]
    fn bench_get_seq_iter(b: &mut Bencher) {
        let n = 100;
        let (_dir, env) = setup_bench_db(n);
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
            assert_eq!(count, n);
        });
    }

}
