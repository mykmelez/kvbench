extern crate tempdir;

use tempdir::TempDir;

fn main() {
    TempDir::new("control").unwrap();
}
