use std::{env, fs, path};
use std::env::current_dir;
use prettyplease::unparse;

use tinkerforge_generator::generator::process_directory;

fn main() {
    parse_json();
}

fn parse_json() {
    let file = process_directory(current_dir().expect("Cannot access current directory").join("bindings"));

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = path::Path::new(&out_dir).join("bindings.rs");
    fs::write(dest_path, unparse(&file)).expect("Cannot write source file");
}
