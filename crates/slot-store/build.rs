use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let shader_dir = manifest_dir.join("../slot-gfx/shaders");
    let config_dir = manifest_dir.join("config/shaders");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let inputs = slot_shader_build::generate_store(&shader_dir, &config_dir, &out_dir)
        .unwrap_or_else(|error| panic!("shader generation failed: {error}"));
    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
}
