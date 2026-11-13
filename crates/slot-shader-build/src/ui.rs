use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Registry;

/// Generate complete UI items from templates, preserving the existing enum
/// and pre-rendering APIs. Non-shader definitions remain in the templates.
pub fn generate_ui(
    shader_dir: &Path,
    template_dir: &Path,
    out_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    let registry_path = shader_dir.join("registry.toml");
    let registry = Registry::load(&registry_path)?;
    let quick_path = template_dir.join("quick_values.rs.in");
    let toast_path = template_dir.join("toasts.rs.in");
    let mut inputs = vec![registry_path, quick_path.clone(), toast_path.clone()];
    inputs.extend(
        registry
            .effects
            .iter()
            .filter_map(|effect| effect.source.as_ref())
            .map(|source| shader_dir.join(source)),
    );

    let mut quick_variants = String::new();
    let mut quick_values = String::new();
    let mut quick_labels = String::new();
    let mut toast_variants = String::new();
    let mut toast_values = String::new();
    let mut toast_labels = String::new();
    for shader in &registry.shaders {
        // Off is shared with non-shader menu settings and declared in the template.
        if shader.id != "Off" {
            writeln!(quick_variants, "    {},", shader.id).unwrap();
            writeln!(quick_values, "        QuickValue::{},", shader.id).unwrap();
        }
        writeln!(
            quick_labels,
            "            QuickValue::{} => {:?},",
            shader.id, shader.label
        )
        .unwrap();
        writeln!(toast_variants, "    Shader{},", shader.id).unwrap();
        writeln!(toast_values, "        Toast::Shader{},", shader.id).unwrap();
        writeln!(
            toast_labels,
            "            Toast::Shader{} => {:?},",
            shader.id,
            format!("Shader: {}", shader.label)
        )
        .unwrap();
    }
    let quick_count = (registry.shaders.len() - 1).to_string();
    let toast_count = registry.shaders.len().to_string();
    let quick = expand(
        &quick_path,
        &[
            ("    // @shader_variants@", quick_variants.trim_end()),
            ("        // @shader_values@", quick_values.trim_end()),
            ("            // @shader_labels@", quick_labels.trim_end()),
            ("@shader_value_count@", &quick_count),
        ],
    )?;
    let toast = expand(
        &toast_path,
        &[
            ("    // @shader_variants@", toast_variants.trim_end()),
            ("        // @shader_toasts@", toast_values.trim_end()),
            ("            // @shader_labels@", toast_labels.trim_end()),
            ("@shader_toast_count@", &toast_count),
        ],
    )?;
    for (name, text) in [("quick_values.rs", quick), ("toasts.rs", toast)] {
        let path = out_dir.join(name);
        fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(inputs)
}

fn expand(path: &Path, replacements: &[(&str, &str)]) -> Result<String, String> {
    let mut text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    for (marker, value) in replacements {
        if text.matches(marker).count() != 1 {
            return Err(format!(
                "{}: expected exactly one {marker:?} marker",
                path.display()
            ));
        }
        text = text.replace(marker, value);
    }
    Ok(format!(
        "// Generated from shaders/registry.toml and UI templates.\n{text}"
    ))
}
