use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Mapping, Registry};

/// Generate direct matches for graphics, menu values, and shortcut toasts.
/// The existing UI variants and labels remain owned by slot-ui.
pub fn generate_app(shader_dir: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let registry_path = shader_dir.join("registry.toml");
    let registry = Registry::load(&registry_path)?;
    let mut inputs = vec![registry_path];
    inputs.extend(
        registry
            .effects
            .iter()
            .filter_map(|effect| effect.source.as_ref())
            .map(|source| shader_dir.join(source)),
    );

    let mut rust = String::from(
        "// Generated from shaders/registry.toml.
impl App {
    pub fn screen_effect(&self) -> slot_gfx::ScreenEffect {
        use slot_gfx::ScreenEffect;
        match self.screen_shader() {
",
    );
    for shader in &registry.shaders {
        match &shader.mapping {
            Mapping::Fixed { effect } => {
                writeln!(
                    rust,
                    "            Shader::{} => ScreenEffect::{effect},",
                    shader.id
                )
                .unwrap();
            }
            Mapping::Lcd { panel_3_2, other } => {
                // Keep ScreenEffect::lcd_for's existing panel predicate and
                // generate its targets from the registry's LCD mapping.
                writeln!(rust, "            Shader::{} => {{", shader.id).unwrap();
                rust.push_str("                let (w, h) = self.panel;\n                match w > 0 && w * 2 == h * 3 {\n");
                writeln!(
                    rust,
                    "                    true => ScreenEffect::{panel_3_2},"
                )
                .unwrap();
                writeln!(rust, "                    false => ScreenEffect::{other},").unwrap();
                rust.push_str("                }\n            }\n");
            }
        }
    }
    rust.push_str("        }\n    }\n}\n\nfn shader_value(shader: Shader) -> QuickValue {\n    match shader {\n");
    for shader in &registry.shaders {
        writeln!(
            rust,
            "        Shader::{} => QuickValue::{},",
            shader.id, shader.id
        )
        .unwrap();
    }
    rust.push_str(
        "    }\n}\n\n#[inline]\nfn shader_toast(shader: Shader) -> Toast {\n    match shader {\n",
    );
    for shader in &registry.shaders {
        writeln!(
            rust,
            "        Shader::{} => Toast::Shader{},",
            shader.id, shader.id
        )
        .unwrap();
    }
    rust.push_str("    }\n}\n");
    let path = out_dir.join("shader_app.rs");
    fs::write(&path, rust).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(inputs)
}
