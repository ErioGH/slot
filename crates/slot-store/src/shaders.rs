use std::sync::OnceLock;

use crate::{Platform, Shader};

/// Ordered built-in shader choices, with an optional 3:2 panel override.
#[derive(Debug)]
pub struct ShaderCatalog {
    available: Vec<Shader>,
    available_3_2: Option<Vec<Shader>>,
}

impl ShaderCatalog {
    /// Parse a bundled catalog, rejecting invalid entries rather than dropping them.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut available = None;
        let mut available_3_2 = None;
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(['#', ';']) {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("line {}: expected key=value", i + 1))?;
            let target = match key.trim() {
                "available" => &mut available,
                "available_3_2" => &mut available_3_2,
                key => return Err(format!("line {}: unknown key {key:?}", i + 1)),
            };
            if target.is_some() {
                return Err(format!("line {}: duplicate key {key:?}", i + 1));
            }
            let mut choices = Vec::new();
            for name in value.split(',').map(str::trim) {
                let shader = Shader::parse(name)
                    .ok_or_else(|| format!("line {}: unknown shader {name:?}", i + 1))?;
                if choices.contains(&shader) {
                    return Err(format!("line {}: duplicate shader {name:?}", i + 1));
                }
                choices.push(shader);
            }
            *target = Some(choices);
        }
        let available = available.ok_or("missing available list")?;
        if let Some(choices) = &available_3_2 {
            for shader in choices {
                if !available.contains(shader) {
                    return Err(format!(
                        "3:2 override contains unavailable shader {:?}",
                        shader.name()
                    ));
                }
            }
        }
        Ok(Self {
            available,
            available_3_2,
        })
    }

    pub fn available(&self) -> &[Shader] {
        &self.available
    }

    /// Panel restrictions do not change which saved selections are supported.
    pub fn supports(&self, shader: Shader) -> bool {
        self.available.contains(&shader)
    }

    pub fn choices(&self, panel: (u32, u32)) -> &[Shader] {
        let (w, h) = panel;
        if w > 0 && u64::from(w) * 2 == u64::from(h) * 3 {
            if let Some(choices) = &self.available_3_2 {
                return choices;
            }
        }
        &self.available
    }
}

/// Embedded catalogs are validated once and shared for the process lifetime.
pub fn shader_catalog(platform: Platform) -> &'static ShaderCatalog {
    static GBAV: OnceLock<ShaderCatalog> = OnceLock::new();
    static GBA: OnceLock<ShaderCatalog> = OnceLock::new();
    static GB: OnceLock<ShaderCatalog> = OnceLock::new();
    static GBC: OnceLock<ShaderCatalog> = OnceLock::new();
    #[cfg(feature = "nds")]
    static NDS: OnceLock<ShaderCatalog> = OnceLock::new();

    if platform == Platform::Gbav {
        return GBAV.get_or_init(|| {
            let gba = shader_catalog(Platform::Gba);
            let filter = |choices: &[Shader]| {
                choices
                    .iter()
                    .copied()
                    .filter(|s| s.supports_gba_video())
                    .collect()
            };
            ShaderCatalog {
                available: filter(&gba.available),
                available_3_2: gba.available_3_2.as_deref().map(filter),
            }
        });
    }
    let (cache, text) = match platform.hardware() {
        Platform::Gba | Platform::Gbav => (&GBA, include_str!("../config/shaders/gba.ini")),
        Platform::Gb => (&GB, include_str!("../config/shaders/gb.ini")),
        Platform::Gbc => (&GBC, include_str!("../config/shaders/gbc.ini")),
        #[cfg(feature = "nds")]
        Platform::Nds => (&NDS, "available=off\n"),
    };
    cache.get_or_init(|| ShaderCatalog::parse(text).expect("invalid embedded shader catalog"))
}
