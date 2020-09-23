extern crate bindgen;
extern crate cmake;

use cmake::Config;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn build_static_cyclonedds() -> String {
    if !Path::new("src/cyclonedds/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init", "src/cyclonedds"])
            .status();
    }

    let dst = Config::new("src/cyclonedds")
        .define("BUILD_IDLC", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=ddsc");

    // Link against OpenSSL libraries
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=crypto");
        println!("cargo:rustc-link-lib=ssl");
    }

    format!("-I{}/include", dst.display())
}

fn main() {
    let bindgen_builder = bindgen::Builder::default()
        .header("wrapper.h")
        .generate_comments(false);

    let bindings = if cfg!(feature = "static") {
        bindgen_builder.clang_arg(build_static_cyclonedds())
    } else {
        println!("cargo:rustc-link-lib=ddsc");
        println!("cargo:rustc-link-search=/opt/ros/foxy/lib/x86_64-linux-gnu");
        println!("cargo:rustc-link-search=/opt/ros/eloquent/lib");
        println!("cargo:rustc-link-search=/opt/ros/dashing/lib");
        bindgen_builder
            .clang_arg("-I/usr/local/include")
            .clang_arg("-I/opt/ros/foxy/include")
            .clang_arg("-I/opt/ros/eloquent/include")
            .clang_arg("-I/opt/ros/dashing/include")
            .clang_arg("-I/usr/lib/gcc/x86_64-linux-gnu/8/include")
    }
    .generate()
    .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
