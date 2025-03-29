// build.rs

use std::path::PathBuf;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    
    // build the kernel, set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    
    // Use bootloader's UefiBoot to create a disk image
    // Note: This assumes bootloader crate is in your dependencies
    // with default-features = false to avoid pulling in ring
    bootloader::UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
}