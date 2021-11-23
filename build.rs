extern crate toml;

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path;

fn main() {
    // Read Cargo.lock and de-toml it
    let mut lock_buf = String::new();
    fs::File::open("Cargo.lock")
        .unwrap()
        .read_to_string(&mut lock_buf)
        .unwrap();
    let lock_toml = toml::Parser::new(&lock_buf).parse().unwrap();

    // Get the table of [[package]]s. This is the deep list of dependencies and dependencies of
    // dependencies.
    let shiromana_rs_version = lock_toml
        .get("package")
        .unwrap()
        .as_slice()
        .unwrap()
        .into_iter()
        .find(|x| x.as_table().unwrap().get("name").unwrap().as_str().unwrap() == "shiromana-rs")
        .unwrap()
        .as_table()
        .unwrap()
        .get("version")
        .unwrap()
        .as_str()
        .unwrap();

    println!("{}", shiromana_rs_version);

    // Write out the file to be included in the module stub
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut versions_file =
        fs::File::create(&path::Path::new(&out_dir).join("versions.include")).unwrap();
    versions_file
        .write(
            format!(
                "pub const SHIROMANA_RS: &'static str = \"{}\";",
                shiromana_rs_version
            )
            .as_ref(),
        )
        .unwrap();
}
