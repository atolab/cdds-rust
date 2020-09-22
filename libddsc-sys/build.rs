extern crate bindgen;
extern crate cmake;

use cmake::Config;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn build_static_cyclonedds(bindgen_builder: &bindgen::Builder) {
    if !Path::new("src/cyclonedds/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init", "src/cyclonedds"])
            .status();
    }

    let dst = Config::new("src/cyclonedds")
        .define("BUILD_IDLC", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        // .define("ENABLE_SSL", "OFF") // Disable SSL for now
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=ddsc");
    dst.display()
}

fn main() {
    let bindgen_builder = bindgen::Builder::default()
        .header("wrapper.h")
        .generate_comments(false);

    let include_paths = if cfg!(feature = "static_cyclonedds") {
        let output_cyclonedds_dir = build_static_cyclonedds(&bindgen_builder);
        vec![format!("-I{}/include", output_cyclonedds_dir)]
    } else {
        println!("cargo:rustc-link-lib=ddsc");
        println!("cargo:rustc-link-search=/opt/ros/foxy/lib/x86_64-linux-gnu");
        println!("cargo:rustc-link-search=/opt/ros/eloquent/lib");
        println!("cargo:rustc-link-search=/opt/ros/dashing/lib");
        vec![
            String::from("/usr/local/include"),
            String::from("/opt/ros/foxy/include"),
            String::from("/opt/ros/eloquent/include"),
            String::from("/opt/ros/dashing/include"),
            String::from("/usr/lib/gcc/x86_64-linux-gnu/8/include"),
        ]
    };

    let bindings = for &include_path in include_paths {
        bindgen_builder.clang_arg(include_path);
    }
    .generate()
    .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
