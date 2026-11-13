use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};

use serde::Deserialize;

/// Ordered registrations, shared by future build-script consumers.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub effects: Vec<Effect>,
    pub shaders: Vec<Shader>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub id: String,
    pub mode: u8,
    /// Relative to the registry's directory; absent only for mode zero.
    pub source: Option<String>,
    pub entry: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    pub id: String,
    /// Stable identifier used in saved state and system catalogs.
    pub name: String,
    pub label: String,
    pub gba_video: bool,
    pub mapping: Mapping,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Mapping {
    Fixed { effect: String },
    Lcd { panel_3_2: String, other: String },
}

impl Registry {
    /// Load and validate registrations and their source files.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let registry = Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        registry.validate(path.parent().unwrap_or(Path::new(".")))?;
        Ok(registry)
    }

    /// Parse the schema. Use `validate` to check references and source files.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    pub fn validate(&self, shader_dir: &Path) -> Result<(), String> {
        if self.effects.is_empty() || self.shaders.is_empty() {
            return Err("registry must contain effects and shaders".into());
        }
        let mut effects = HashSet::new();
        let mut modes = HashSet::new();
        let mut sources = HashSet::new();
        let mut entries = HashSet::new();
        for effect in &self.effects {
            variant(&effect.id)?;
            unique(&mut effects, &effect.id, "effect id")?;
            if !modes.insert(effect.mode) {
                return Err(format!("duplicate GPU mode {}", effect.mode));
            }
            if effect.mode == 0 {
                if effect.id != "None" || effect.source.is_some() || effect.entry.is_some() {
                    return Err("mode zero must be None without a source or entry point".into());
                }
                continue;
            }
            if effect.id == "None" {
                return Err("None must use GPU mode zero".into());
            }
            let source = effect
                .source
                .as_deref()
                .ok_or_else(|| format!("{}: missing source", effect.id))?;
            let path = Path::new(source);
            if source.is_empty()
                || path.extension().is_none_or(|ext| ext != "glsl")
                || path
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                return Err(format!(
                    "{}: source must be a relative GLSL path",
                    effect.id
                ));
            }
            unique(&mut sources, source, "source")?;
            let entry = effect
                .entry
                .as_deref()
                .ok_or_else(|| format!("{}: missing entry point", effect.id))?;
            if !entry.starts_with("effect_") || !identifier(entry) {
                return Err(format!("{}: invalid entry point {entry:?}", effect.id));
            }
            unique(&mut entries, entry, "entry point")?;
            let path = shader_dir.join(path);
            fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        if !effects.contains("None") {
            return Err("missing None effect".into());
        }

        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        let mut referenced = HashSet::new();
        for shader in &self.shaders {
            variant(&shader.id)?;
            unique(&mut ids, &shader.id, "shader id")?;
            if shader.name.is_empty()
                || shader.name.split('-').any(|part| {
                    part.is_empty()
                        || !part
                            .bytes()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                })
            {
                return Err(format!(
                    "{}: invalid saved name {:?}",
                    shader.id, shader.name
                ));
            }
            unique(&mut names, &shader.name, "saved name")?;
            if shader.label.trim().is_empty() || shader.label.chars().any(char::is_control) {
                return Err(format!("{}: invalid label", shader.id));
            }
            let targets = match &shader.mapping {
                Mapping::Fixed { effect } => vec![effect],
                Mapping::Lcd { panel_3_2, other } => {
                    if panel_3_2 == other || panel_3_2 == "None" || other == "None" {
                        return Err(format!("{}: LCD requires two distinct effects", shader.id));
                    }
                    vec![panel_3_2, other]
                }
            };
            for target in targets {
                if !effects.contains(target.as_str()) {
                    return Err(format!("{}: unknown effect {target:?}", shader.id));
                }
                if (target == "None") != (shader.id == "Off") {
                    return Err("only Off may map to None, and Off must map to None".into());
                }
                referenced.insert(target.as_str());
            }
            if shader.id == "Off" && (shader.name != "off" || !shader.gba_video) {
                return Err("Off must have saved name off and support GBA Video".into());
            }
        }
        if !ids.contains("Off") {
            return Err("missing Off shader".into());
        }
        for effect in &self.effects {
            if !referenced.contains(effect.id.as_str()) {
                return Err(format!("{}: effect is not mapped by any shader", effect.id));
            }
        }
        Ok(())
    }

    /// Validate a system catalog without changing its availability or order.
    pub fn validate_catalog(&self, text: &str) -> Result<(), String> {
        let mut available = None;
        let mut available_3_2 = None;
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(['#', ';']) {
                continue;
            }
            let line_number = i + 1;
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("line {line_number}: expected key=value"))?;
            let target = match key.trim() {
                "available" => &mut available,
                "available_3_2" => &mut available_3_2,
                key => return Err(format!("line {line_number}: unknown key {key:?}")),
            };
            if target.is_some() {
                return Err(format!("line {line_number}: duplicate key {key:?}"));
            }
            let mut choices = HashSet::new();
            for name in value.split(',').map(str::trim) {
                if !self.shaders.iter().any(|s| s.name == name) {
                    return Err(format!("line {line_number}: unknown shader {name:?}"));
                }
                if !choices.insert(name) {
                    return Err(format!("line {line_number}: duplicate shader {name:?}"));
                }
            }
            *target = Some(choices);
        }
        let available = available.ok_or("missing available list")?;
        if let Some(choices) = available_3_2 {
            if !choices.is_subset(&available) {
                return Err("3:2 override contains unavailable shader".into());
            }
        }
        Ok(())
    }

    /// Check every INI in the system shader catalog directory.
    pub fn validate_catalogs(&self, config_dir: &Path) -> Result<(), String> {
        let mut paths = fs::read_dir(config_dir)
            .and_then(|entries| {
                entries
                    .map(|e| e.map(|e| e.path()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .map_err(|e| format!("{}: {e}", config_dir.display()))?;
        paths.sort();
        for path in paths {
            if path.extension().is_some_and(|ext| ext == "ini") {
                let text =
                    fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                self.validate_catalog(&text)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
        Ok(())
    }
}

fn identifier(value: &str) -> bool {
    value.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

fn variant(value: &str) -> Result<(), String> {
    if !identifier(value) || !value.starts_with(|c: char| c.is_ascii_uppercase()) || value == "Self"
    {
        return Err(format!("invalid Rust variant {value:?}"));
    }
    Ok(())
}

fn unique<'a>(set: &mut HashSet<&'a str>, value: &'a str, kind: &str) -> Result<(), String> {
    if !set.insert(value) {
        return Err(format!("duplicate {kind} {value:?}"));
    }
    Ok(())
}
