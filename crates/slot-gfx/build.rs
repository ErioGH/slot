use std::path::PathBuf;

fn main() {
    let shader_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("shaders");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let inputs = slot_shader_build::generate_gfx(&shader_dir, &out_dir)
        .unwrap_or_else(|error| panic!("shader generation failed: {error}"));
    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
}
