[![Build Status](https://travis-ci.org/mykmelez/kvbench.svg?branch=master)](https://travis-ci.org/mykmelez/kvbench)

# kvbench

This repo contains a set of benchmarks for the LMDB and LevelDB key-value storage engines.  Its purpose is to provide useful metrics on the relative performance of the two engines.  It may also prove useful for comparing disk footprint (of both the engines themselves and their storage files), memory footprint, reliability, and developer ergonomics.

The benchmarks are written using [Criterion.rs](https://japaric.github.io/criterion.rs/book/), a "statistics-driven micro-benchmarking tool."

# Use

```sh
cargo bench
```

# Example

Here's part of the output of an example benchmark run (edited for brevity):

```
> cargo bench
…
     Running target/release/deps/compare-53608677cd816849
cmp_open_db/leveldb     time:   [1.3044 ms 1.3136 ms 1.3251 ms]
…
cmp_open_db/lmdb        time:   [165.57 us 166.70 us 167.94 us]
…
leveldb_put_seq_sync/(1, 1)
                        time:   [56.046 us 56.619 us 57.429 us]
…
leveldb_put_seq_sync/(1, 100)
                        time:   [57.474 us 58.370 us 59.578 us]
…
leveldb_db_size/(1000, 1000)
                        time:   [1.6267 ms 1.6330 ms 1.6386 ms]
…
     Running target/release/deps/lmdb-93c20684e4f1f806
lmdb_open_db            time:   [162.39 us 163.03 us 163.75 us]
…
lmdb_put_seq_sync/(1, 1)
                        time:   [106.53 us 107.16 us 107.91 us]
…
lmdb_put_seq_sync/(1, 100)
                        time:   [106.70 us 107.21 us 107.72 us]
…
lmdb_db_size/(1000, 1000)
```

The tuples in test names are combinations of the number of pairs of keys/values and the sizes of the values. For example, the test named "lmdb_put_seq_sync/(1, 100)" writes a single key/value pair to the datastore, and the value size is 100 bytes.

Note that the "db_size" benches are a measure of space, not time.  They reflect the size of storage files into elapsed time via a hack (sleeping for the file size in bytes's number of nanoseconds).  This may or may not be a reasonable way to measure the disk footprint of storage files.

Here's an example of the relative disk footprint (in kilobytes) of the benchmarking programs, compared to a control program:

```
> cargo build --release && ls -1sk target/release/{control,leveldb,lmdb}
…
 608 target/release/control
 864 target/release/leveldb
 684 target/release/lmdb
```

Also see this [example Criterion.rs report](https://mykmelez.github.io/kvbench/criterion/report/).

# Caveats

A limitation of the approach is that the benchmarks rely on Rust wrappers—the [lmdb](https://github.com/danburkert/lmdb-rs) and [leveldb](https://crates.io/crates/leveldb) crates—rather than calling into the C/C++ storage engine libraries directly, so overhead in the wrappers will incorrectly accrue to the underlying libraries.
