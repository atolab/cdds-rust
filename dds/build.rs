extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=ddsc");
    println!("cargo:rustc-link-search=/opt/ros/foxy/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=/opt/ros/eloquent/lib");
    println!("cargo:rustc-link-search=/opt/ros/dashing/lib");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/local/include")
        .clang_arg("-I/opt/ros/foxy/include")
        .clang_arg("-I/opt/ros/eloquent/include")
        .clang_arg("-I/opt/ros/dashing/include")
        .clang_arg("-I/usr/lib/gcc/x86_64-linux-gnu/8/include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
