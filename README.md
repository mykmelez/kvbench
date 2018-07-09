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
     Running target/release/deps/leveldb-a8f0442a215d9284

running 9 tests
test tests::bench_db_size        ... bench:       6,695 ns/iter (+/- 492)
test tests::bench_get_rand       ... bench:     145,389 ns/iter (+/- 41,841)
test tests::bench_get_seq        ... bench:     140,288 ns/iter (+/- 31,929)
test tests::bench_get_seq_iter   ... bench:      15,800 ns/iter (+/- 4,328)
test tests::bench_open_db        ... bench:   1,563,049 ns/iter (+/- 162,092)
test tests::bench_put_rand_async ... bench:     316,041 ns/iter (+/- 35,024)
test tests::bench_put_rand_sync  ... bench:     135,399 ns/iter (+/- 35,718)
test tests::bench_put_seq_async  ... bench:     122,410 ns/iter (+/- 19,453)
test tests::bench_put_seq_sync   ... bench:     302,608 ns/iter (+/- 65,318)

test result: ok. 0 passed; 0 failed; 0 ignored; 9 measured; 0 filtered out

     Running target/release/deps/lmdb-84a26ec164f48495

running 9 tests
test tests::bench_db_size        ... bench:      33,051 ns/iter (+/- 4,556)
test tests::bench_get_rand       ... bench:       8,239 ns/iter (+/- 1,764)
test tests::bench_get_seq        ... bench:       8,422 ns/iter (+/- 2,781)
test tests::bench_get_seq_iter   ... bench:       1,501 ns/iter (+/- 476)
test tests::bench_open_db        ... bench:     204,231 ns/iter (+/- 38,441)
test tests::bench_put_rand_async ... bench:      86,203 ns/iter (+/- 15,408)
test tests::bench_put_rand_sync  ... bench:     160,896 ns/iter (+/- 64,842)
test tests::bench_put_seq_async  ... bench:      86,351 ns/iter (+/- 28,824)
test tests::bench_put_seq_sync   ... bench:     149,998 ns/iter (+/- 32,966)
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
