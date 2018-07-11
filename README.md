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
     Running target/release/deps/leveldb-1abc2c24f9c191cf
leveldb_open_db         time:   [1.3582 ms 1.3711 ms 1.3865 ms]
                        change: [-10.744% -8.2753% -5.8404%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe

leveldb_put_seq_sync/1  time:   [55.192 us 56.010 us 57.236 us]
                        change: [+0.1947% +2.6007% +5.3932%] (p = 0.04 < 0.05)
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  6 (6.00%) high severe
leveldb_put_seq_sync/128
                        time:   [377.23 us 397.50 us 421.24 us]
                        change: [-22.371% -16.567% -10.286%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  12 (12.00%) high severe
leveldb_put_seq_sync/1024
                        time:   [1.2903 ms 1.2990 ms 1.3088 ms]
                        change: [-45.079% -42.306% -39.356%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild

…

leveldb_get_seq/1       time:   [1.0293 us 1.0366 us 1.0451 us]
                        change: [-1.7149% -0.1274% +1.5447%] (p = 0.88 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
leveldb_get_seq/128     time:   [158.59 us 159.77 us 160.96 us]
                        change: [-1.6600% +0.2003% +1.9977%] (p = 0.84 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
leveldb_get_seq/1024    time:   [1.3232 ms 1.3337 ms 1.3457 ms]
                        change: [-0.0770% +1.5917% +3.5941%] (p = 0.08 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

…

leveldb_db_size/1       time:   [8.0406 us 8.2096 us 8.3724 us]
                        change: [-0.9352% +1.7033% +4.4698%] (p = 0.22 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
leveldb_db_size/128     time:   [7.8870 us 8.0797 us 8.2627 us]
                        change: [+3.7505% +6.8143% +9.9680%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
leveldb_db_size/1024    time:   [24.231 us 24.426 us 24.616 us]
                        change: [+0.4283% +1.5960% +2.7463%] (p = 0.01 < 0.05)
                        Change within noise threshold.

```

Note that the "db_size" benches are a measure of space, not time.  They reflect the size of storage files into elapsed time via a hack (sleeping for the file size in bytes's number of nanoseconds).  This may or may not be a reasonable way to measure the disk footprint of storage files.

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
