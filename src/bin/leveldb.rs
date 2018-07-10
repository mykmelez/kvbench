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

#![feature(test)]

extern crate db_key;
extern crate leveldb;
extern crate tempdir;

use db_key::Key;
use leveldb::database::batch::{
    Batch,
    Writebatch,
};
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{
    Options,
    ReadOptions,
    WriteOptions,
};
use tempdir::TempDir;

fn main() {
    let dir = TempDir::new("example").unwrap();
    let mut options = Options::new();
    options.create_if_missing = true;
    let database: Database<i32> = Database::open(dir.path(), options).unwrap();

    let key1 = Key::from_u8(b"key1");
    let key2 = Key::from_u8(b"key2");
    let key3 = Key::from_u8(b"key3");

    let batch = &mut Writebatch::new();
    batch.put(key1, b"val1");
    batch.put(key2, b"val2");
    batch.put(key3, b"val3");
    database.write(WriteOptions::new(), batch).unwrap();

    assert_eq!(database.get(ReadOptions::new(), key1).unwrap().unwrap(), b"val1");
    assert_eq!(database.get(ReadOptions::new(), key2).unwrap().unwrap(), b"val2");
    assert_eq!(database.get(ReadOptions::new(), key3).unwrap().unwrap(), b"val3");

    database.delete(WriteOptions::new(), key1).unwrap();
    assert_eq!(database.get(ReadOptions::new(), key1).unwrap(), None);
}
