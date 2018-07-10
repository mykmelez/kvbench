[![Build Status](https://travis-ci.org/mykmelez/kv-bench.svg?branch=master)](https://travis-ci.org/mykmelez/kv-bench)

# kv-bench

This repo contains a set of benchmarks for the LMDB and LevelDB key-value storage engines.  Its purpose is to provide useful metrics on the relative performance of the two engines.  It may also prove useful for comparing disk footprint (of both the engines themselves and their storage files), memory footprint, reliability, and developer ergonomics.

The benchmarks are written using the Rust language's built-in (albeit unstable) [support for benchmark testing](https://doc.rust-lang.org/unstable-book/library-features/test.html).

# Use

```sh
cargo +nightly bench
```

# Example

Here's an example benchmark run:

```
     Running target/release/deps/leveldb-22310b491fd3091b

running 9 tests
test tests::bench_db_size        ... bench:       7,213 ns/iter (+/- 2,936)
test tests::bench_get_rand       ... bench:     121,786 ns/iter (+/- 10,359)
test tests::bench_get_seq        ... bench:     119,470 ns/iter (+/- 11,986)
test tests::bench_get_seq_iter   ... bench:      13,218 ns/iter (+/- 1,894)
test tests::bench_open_db        ... bench:   1,357,889 ns/iter (+/- 299,411)
test tests::bench_put_rand_async ... bench:     114,261 ns/iter (+/- 15,022)
test tests::bench_put_rand_sync  ... bench:     361,174 ns/iter (+/- 267,127)
test tests::bench_put_seq_async  ... bench:     107,181 ns/iter (+/- 11,718)
test tests::bench_put_seq_sync   ... bench:     353,104 ns/iter (+/- 297,763)

test result: ok. 0 passed; 0 failed; 0 ignored; 9 measured; 0 filtered out

     Running target/release/deps/lmdb-5c56392ee85c8251

running 9 tests
test tests::bench_db_size        ... bench:      32,830 ns/iter (+/- 4,925)
test tests::bench_get_rand       ... bench:       6,867 ns/iter (+/- 408)
test tests::bench_get_seq        ... bench:       6,784 ns/iter (+/- 681)
test tests::bench_get_seq_iter   ... bench:       1,278 ns/iter (+/- 65)
test tests::bench_open_db        ... bench:     161,971 ns/iter (+/- 13,359)
test tests::bench_put_rand_async ... bench:      73,461 ns/iter (+/- 5,791)
test tests::bench_put_rand_sync  ... bench:     122,451 ns/iter (+/- 20,576)
test tests::bench_put_seq_async  ... bench:      73,005 ns/iter (+/- 2,730)
test tests::bench_put_seq_sync   ... bench:     125,105 ns/iter (+/- 10,181)
```

Note that "db_size" bench is a measure of space, not time.  It reflects the size of storage files into elapsed time via a hack (sleeping for the file size in bytes's number of nanoseconds).  This may or may not be a reasonable way to measure the disk footprint of storage files.

Here's an example of the relative disk footprint (in kilobytes) of the benchmarking programs, compared to a control program:

```sh
> cargo +nightly build --release && ls -1sk target/release/
 608 control
 860 leveldb
 684 lmdb
```

# Caveats

A limitation of the approach is that the benchmarks rely on Rust wrappers—the [lmdb](https://github.com/danburkert/lmdb-rs) and [leveldb](https://crates.io/crates/leveldb) crates—rather than calling into the C/C++ storage engine libraries directly, so overhead in the wrappers will incorrectly accrue to the underlying libraries.
