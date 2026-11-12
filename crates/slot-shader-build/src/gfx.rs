use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Registry;

/// Generate graphics registration in `out_dir`, returning all file inputs for
/// the caller's `cargo:rerun-if-changed` directives. Effect bodies are copied verbatim.
pub fn generate_gfx(shader_dir: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let registry_path = shader_dir.join("registry.toml");
    let template_path = shader_dir.join("game.frag.glsl");
    let registry = Registry::load(&registry_path)?;
    let template = read(&template_path)?;
    let mut inputs = vec![registry_path, template_path.clone()];

    // Explicit discriminants keep GPU modes independent of registry/source order.
    let mut by_mode: Vec<_> = registry.effects.iter().collect();
    by_mode.sort_by_key(|effect| effect.mode);
    let mut rust = String::from(
        "// Generated from shaders/registry.toml.\n\
         #[derive(Copy, Clone, PartialEq, Eq, Debug)]\n\
         pub enum ScreenEffect {\n",
    );
    for effect in &by_mode {
        writeln!(rust, "    {} = {},", effect.id, effect.mode).unwrap();
    }
    rust.push_str("}\n");

    let mut declarations = String::new();
    for effect in &registry.effects {
        if let Some(entry) = &effect.entry {
            writeln!(declarations, "void {entry}(inout vec3 c, vec2 uv, vec2 f);").unwrap();
        }
    }
    let mut dispatch = String::new();
    for effect in by_mode.into_iter().rev() {
        if let Some(entry) = &effect.entry {
            writeln!(
                dispatch,
                "    }} else if (u_mode > {}.5) {{",
                effect.mode - 1
            )
            .unwrap();
            writeln!(dispatch, "        {entry}(c, uv, f);").unwrap();
        }
    }
    let mut frag = replace_marker(
        &template,
        "// @effect_declarations@",
        declarations.trim_end(),
        &template_path,
    )?;
    frag = replace_marker(
        &frag,
        "    // @effect_dispatch@",
        dispatch.trim_end(),
        &template_path,
    )?;
    for effect in &registry.effects {
        if let Some(source) = &effect.source {
            let path = shader_dir.join(source);
            frag.push_str(&read(&path)?);
            inputs.push(path);
        }
    }

    write(&out_dir.join("screen_effect.rs"), &rust)?;
    write(&out_dir.join("game.frag.glsl"), &frag)?;
    Ok(inputs)
}

fn replace_marker(text: &str, marker: &str, value: &str, path: &Path) -> Result<String, String> {
    if text.matches(marker).count() != 1 {
        return Err(format!(
            "{}: expected exactly one {marker:?} marker",
            path.display()
        ));
    }
    Ok(text.replace(marker, value))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
