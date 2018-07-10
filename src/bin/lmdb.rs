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
    let dir = TempDir::new("example").unwrap();
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
