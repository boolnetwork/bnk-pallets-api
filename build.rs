pub fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed={crate_dir}/metadata.scale");
}
