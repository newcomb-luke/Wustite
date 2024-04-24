use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    println!("cargo:rustc-link-arg=/debug:dwarf");
    println!(
        "cargo:rustc-link-arg=-Map={}",
        local_path.join("bootloader-uefi.map").display()
    );
}
