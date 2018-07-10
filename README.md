[![Build Status](https://travis-ci.org/mykmelez/kv-bench.svg?branch=master)](https://travis-ci.org/mykmelez/kv-bench)

# kv-bench

This repo contains a set of benchmarks for the LMDB and LevelDB key-value storage engines.  Its purpose is to provide useful metrics on the relative performance of the two engines.  It may also prove useful for comparing disk footprint (of both the engines themselves and their storage files), memory footprint, reliability, and developer ergonomics.

The benchmarks are written using the [criterion](https://docs.rs/criterion) Rust benchmarking crate.

# Use

```sh
cargo bench
```

# Example

Here's part of the output of an example benchmark run (edited for brevity):

```
> cargo bench
…
     Running target/release/deps/leveldb-868d3dbdca02a560
Gnuplot not found, disabling plotting
leveldb_open_db         time:   [1.3583 ms 1.3722 ms 1.3883 ms]
                        change: [-1.5554% -0.3191% +0.9426%] (p = 0.63 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

leveldb_put_seq_sync    time:   [344.63 us 357.80 us 374.31 us]
                        change: [-8.9291% -4.3827% +0.3533%] (p = 0.07 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  4 (4.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe

…

leveldb_get_seq         time:   [118.56 us 119.03 us 119.53 us]
                        change: [-3.4437% -2.5659% -1.6572%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

…

leveldb_db_size         time:   [7.6352 us 7.8282 us 8.0229 us]
                        change: [-1.4846% +1.9984% +5.7711%] (p = 0.28 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
```

Note that "db_size" bench is a measure of space, not time.  It reflects the size of storage files into elapsed time via a hack (sleeping for the file size in bytes's number of nanoseconds).  This may or may not be a reasonable way to measure the disk footprint of storage files.

Here's an example of the relative disk footprint (in kilobytes) of the benchmarking programs, compared to a control program:

```
> cargo build --release && ls -1sk target/release/{control,leveldb,lmdb}
…
 608 target/release/control
 864 target/release/leveldb
 684 target/release/lmdb
```

# Caveats

A limitation of the approach is that the benchmarks rely on Rust wrappers—the [lmdb](https://github.com/danburkert/lmdb-rs) and [leveldb](https://crates.io/crates/leveldb) crates—rather than calling into the C/C++ storage engine libraries directly, so overhead in the wrappers will incorrectly accrue to the underlying libraries.
