use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path.join("linker.ld").display()
    );
    println!(
        "cargo:rustc-link-arg=-Map={}",
        local_path.join("bootloader.map").display()
    );
    println!("cargo:rustc-link-search={}", local_path.display());
    println!("cargo:rustc-link-lib=entry");
}
