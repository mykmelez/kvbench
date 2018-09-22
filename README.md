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
cmp_open_db/leveldb     time:   [1.3823 ms 1.3895 ms 1.3981 ms]
…
cmp_open_db/lmdb        time:   [165.57 us 166.70 us 167.94 us]
…
leveldb_put_seq_sync/Param { num_pairs: 1, size_values: 1 }
                        time:   [57.065 us 58.691 us 61.182 us]
…
leveldb_put_seq_sync/Param { num_pairs: 1, size_values: 100 }
                        time:   [62.209 us 63.279 us 64.509 us]
…
leveldb_db_size/Param { num_pairs: 1000, size_values: 1000 }
                        time:   [1.5824 ms 1.5971 ms 1.6097 ms]
…
     Running target/release/deps/lmdb-93c20684e4f1f806
lmdb_open_db            time:   [162.25 us 162.90 us 163.59 us]
…
lmdb_put_seq_sync/Param { num_pairs: 1, size_values: 1 }
                        time:   [113.81 us 114.74 us 115.72 us]
…
lmdb_put_seq_sync/Param { num_pairs: 1, size_values: 100 }
                        time:   [112.52 us 116.05 us 123.21 us]
…
lmdb_db_size/Param { num_pairs: 1000, size_values: 1000 }
                        time:   [1.8733 ms 1.8784 ms 1.8826 ms]
```

The tuples in test names are combinations of the number of pairs of keys/values and the sizes of the values. For example, the test named "lmdb_put_seq_sync/Param { num_pairs: 1, size_values: 100 }" writes a single key/value pair to the datastore, and the size of the value is 100 bytes.

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
