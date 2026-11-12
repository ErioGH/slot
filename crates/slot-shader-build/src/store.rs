use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Registry;

/// Generate saved shader identities and GBA Video eligibility. System INIs
/// remain the authority for availability and cycling order.
pub fn generate_store(
    shader_dir: &Path,
    config_dir: &Path,
    out_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    let registry_path = shader_dir.join("registry.toml");
    let registry = Registry::load(&registry_path)?;
    registry.validate_catalogs(config_dir)?;
    let mut inputs = vec![registry_path, config_dir.to_path_buf()];
    inputs.extend(
        registry
            .effects
            .iter()
            .filter_map(|effect| effect.source.as_ref())
            .map(|source| shader_dir.join(source)),
    );

    let mut rust = String::from(
        "// Generated from shaders/registry.toml.\n\
         #[derive(Copy, Clone, PartialEq, Eq, Debug)]\n\
         pub enum Shader {\n",
    );
    for shader in &registry.shaders {
        writeln!(rust, "    {},", shader.id).unwrap();
    }
    rust.push_str(
        "}\n\nimpl Shader {\n    pub fn name(self) -> &'static str {\n        match self {\n",
    );
    for shader in &registry.shaders {
        writeln!(
            rust,
            "            Shader::{} => {:?},",
            shader.id, shader.name
        )
        .unwrap();
    }
    rust.push_str("        }\n    }\n\n    pub fn parse(name: &str) -> Option<Shader> {\n        match name {\n");
    for shader in &registry.shaders {
        writeln!(
            rust,
            "            {:?} => Some(Shader::{}),",
            shader.name, shader.id
        )
        .unwrap();
    }
    rust.push_str("            _ => None,\n        }\n    }\n\n    pub(crate) fn supports_gba_video(self) -> bool {\n        match self {\n");
    for shader in &registry.shaders {
        writeln!(
            rust,
            "            Shader::{} => {},",
            shader.id, shader.gba_video
        )
        .unwrap();
    }
    rust.push_str("        }\n    }\n}\n");
    let path = out_dir.join("shader.rs");
    fs::write(&path, rust).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(inputs)
}
