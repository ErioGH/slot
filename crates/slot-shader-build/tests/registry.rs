use std::fs;
use std::path::PathBuf;

use slot_shader_build::{Mapping, Registry};

const REGISTRY: &str = include_str!("../../slot-gfx/shaders/registry.toml");

fn shader_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slot-gfx/shaders")
}

fn registry() -> Registry {
    Registry::parse(REGISTRY).unwrap()
}

fn rejects(change: impl FnOnce(&mut Registry), message: &str) {
    let mut registry = registry();
    change(&mut registry);
    let error = registry.validate(&shader_dir()).unwrap_err();
    assert!(
        error.contains(message),
        "expected {message:?}, got {error:?}"
    );
}

#[test]
fn bundled_registry_and_system_catalogs_are_valid() {
    let registry = Registry::load(&shader_dir().join("registry.toml")).unwrap();
    registry
        .validate_catalogs(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slot-store/config/shaders"),
        )
        .unwrap();
}

#[test]
fn saved_names_labels_and_gba_video_eligibility_are_preserved() {
    let registry = registry();
    let shaders: Vec<_> = registry
        .shaders
        .iter()
        .map(|s| {
            (
                s.id.as_str(),
                s.name.as_str(),
                s.label.as_str(),
                s.gba_video,
            )
        })
        .collect();
    assert_eq!(
        shaders,
        [
            ("Off", "off", "Off", true),
            ("Lcd3x", "lcd3x", "LCD", true),
            ("Grid", "grid", "Grid", true),
            ("Dot", "dot", "Dot", true),
            ("Simpletex", "simpletex", "Simpletex", true),
            ("Shimmerless", "shimmerless", "Shimmerless", true),
            (
                "DarkGrittyRetro",
                "dark-gritty-retro",
                "Dark Gritty Retro",
                false
            ),
        ]
    );
}

#[test]
fn graphics_modes_and_source_assembly_order_are_preserved() {
    let registry = registry();
    let effects: Vec<_> = registry
        .effects
        .iter()
        .map(|e| {
            (
                e.id.as_str(),
                e.mode,
                e.source.as_deref(),
                e.entry.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        effects,
        [
            ("None", 0, None, None),
            (
                "DarkGrittyRetro",
                7,
                Some("effects/dark_gritty_retro.glsl"),
                Some("effect_dark_gritty_retro")
            ),
            (
                "LcdGrid",
                6,
                Some("effects/lcd_grid.glsl"),
                Some("effect_lcd_grid")
            ),
            (
                "Shimmerless",
                5,
                Some("effects/shimmerless.glsl"),
                Some("effect_shimmerless")
            ),
            (
                "Simpletex",
                4,
                Some("effects/simpletex.glsl"),
                Some("effect_simpletex")
            ),
            ("Dot", 3, Some("effects/dot.glsl"), Some("effect_dot")),
            ("Grid", 2, Some("effects/grid.glsl"), Some("effect_grid")),
            ("Lcd3x", 1, Some("effects/lcd3x.glsl"), Some("effect_lcd3x")),
        ]
    );
}

#[test]
fn off_and_panel_dependent_lcd_are_separate_from_implementations() {
    let registry = registry();
    assert_eq!(
        registry.shaders[0].mapping,
        Mapping::Fixed {
            effect: "None".into()
        }
    );
    assert_eq!(
        registry.shaders[1].mapping,
        Mapping::Lcd {
            panel_3_2: "Lcd3x".into(),
            other: "LcdGrid".into()
        }
    );
    for shader in &registry.shaders[2..] {
        assert_eq!(
            shader.mapping,
            Mapping::Fixed {
                effect: shader.id.clone()
            }
        );
    }
    assert!(!registry.shaders.iter().any(|s| s.id == "LcdGrid"));
}

#[test]
fn duplicate_ids_names_and_modes_are_rejected() {
    rejects(|r| r.effects[1].id = "None".into(), "duplicate effect id");
    rejects(|r| r.shaders[1].id = "Off".into(), "duplicate shader id");
    rejects(|r| r.shaders[1].name = "off".into(), "duplicate saved name");
    rejects(|r| r.effects[1].mode = 0, "duplicate GPU mode");
}

#[test]
fn duplicate_sources_and_entry_points_are_rejected() {
    rejects(
        |r| r.effects[2].source = r.effects[1].source.clone(),
        "duplicate source",
    );
    rejects(
        |r| r.effects[2].entry = r.effects[1].entry.clone(),
        "duplicate entry point",
    );
}

#[test]
fn source_files_must_exist_and_be_readable() {
    rejects(
        |r| r.effects[1].source = Some("effects/missing.glsl".into()),
        "effects/missing.glsl",
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("registry.toml");
    fs::write(&path, REGISTRY).unwrap();
    let error = Registry::load(&path).unwrap_err();
    assert!(error.contains("effects/dark_gritty_retro.glsl"), "{error}");
}

#[test]
fn implementations_require_relative_glsl_sources_and_entry_points() {
    rejects(|r| r.effects[1].source = None, "missing source");
    rejects(|r| r.effects[1].entry = None, "missing entry point");
    rejects(
        |r| r.effects[1].entry = Some("bad()".into()),
        "invalid entry point",
    );
    for path in [
        "",
        "/tmp/shader.glsl",
        "../shader.glsl",
        "effects/shader.txt",
    ] {
        rejects(
            |r| r.effects[1].source = Some(path.into()),
            "relative GLSL path",
        );
    }
}

#[test]
fn unknown_and_unused_effects_are_rejected() {
    rejects(
        |r| {
            r.shaders[2].mapping = Mapping::Fixed {
                effect: "Missing".into(),
            }
        },
        "unknown effect",
    );
    rejects(
        |r| {
            r.shaders[1].mapping = Mapping::Lcd {
                panel_3_2: "Missing".into(),
                other: "LcdGrid".into(),
            }
        },
        "unknown effect",
    );
    rejects(
        |r| {
            r.shaders.pop();
        },
        "effect is not mapped",
    );
}

#[test]
fn lcd_requires_two_distinct_rendering_implementations() {
    for other in ["Lcd3x", "None"] {
        rejects(
            |r| {
                r.shaders[1].mapping = Mapping::Lcd {
                    panel_3_2: "Lcd3x".into(),
                    other: other.into(),
                }
            },
            "LCD requires two distinct effects",
        );
    }
}

#[test]
fn off_is_the_only_selection_without_an_implementation() {
    rejects(
        |r| r.effects[0].source = Some("effects/off.glsl".into()),
        "mode zero",
    );
    rejects(
        |r| r.effects[0].entry = Some("effect_off".into()),
        "mode zero",
    );
    rejects(|r| r.effects[0].mode = 8, "None must use GPU mode zero");
    rejects(
        |r| {
            r.effects.remove(0);
        },
        "missing None effect",
    );
    rejects(
        |r| {
            r.shaders.remove(0);
        },
        "missing Off shader",
    );
    rejects(
        |r| {
            r.shaders[0].mapping = Mapping::Fixed {
                effect: "Grid".into(),
            }
        },
        "Off must map to None",
    );
    rejects(
        |r| {
            r.shaders[2].mapping = Mapping::Fixed {
                effect: "None".into(),
            }
        },
        "only Off may map to None",
    );
    rejects(
        |r| r.shaders[0].gba_video = false,
        "Off must have saved name off",
    );
    rejects(
        |r| r.shaders[0].name = "disabled".into(),
        "Off must have saved name off",
    );
}

#[test]
fn invalid_identifiers_names_labels_and_empty_registries_are_rejected() {
    for id in ["", "bad-id", "1Shader", "Self"] {
        rejects(|r| r.shaders[1].id = id.into(), "invalid Rust variant");
    }
    for name in ["", "Bad", "bad_name", "bad--name"] {
        rejects(|r| r.shaders[1].name = name.into(), "invalid saved name");
    }
    for label in [" ", "LCD\n"] {
        rejects(|r| r.shaders[1].label = label.into(), "invalid label");
    }
    rejects(|r| r.shaders.clear(), "must contain effects and shaders");
    rejects(|r| r.effects.clear(), "must contain effects and shaders");
}

#[test]
fn malformed_schema_and_unknown_fields_are_rejected() {
    for text in [
        "not toml",
        &REGISTRY.replace("gba_video = true", "gba_vdeo = true"),
        &REGISTRY.replace("kind = \"fixed\"", "kind = \"unknown\""),
        &REGISTRY.replace("mode = 7", "mode = 256"),
        &REGISTRY.replace("kind = \"lcd\"", "kind = \"lcd\", unexpected = true"),
        &REGISTRY.replace("label = \"LCD\"\n", ""),
    ] {
        assert!(Registry::parse(text).is_err(), "accepted {text}");
    }
}

#[test]
fn catalogs_accept_comments_whitespace_and_configured_order() {
    registry()
        .validate_catalog(
            "\n# comment\n; comment\n available = dot , off , grid\navailable_3_2=grid,dot\n",
        )
        .unwrap();
    // NDS remains an inline Off-only catalog in slot-store.
    registry().validate_catalog("available=off\n").unwrap();
}

#[test]
fn unknown_catalog_names_are_rejected_in_both_lists() {
    for text in [
        "available=off,missing",
        "available=off\navailable_3_2=missing",
    ] {
        assert!(registry()
            .validate_catalog(text)
            .unwrap_err()
            .contains("unknown shader"));
    }
    // Graphics-only implementations are not saved shader choices.
    assert!(registry().validate_catalog("available=lcd-grid").is_err());
}

#[test]
fn invalid_catalog_structure_is_rejected() {
    for text in [
        "",
        "available_3_2=off",
        "available=",
        "available=off,",
        "available=off\navailable_3_2=",
        "available=off,off",
        "available=off\navailable_3_2=off,off",
        "available=off\navailable_3_2=grid",
        "available off",
        "availble=off",
        "available=off\navailable=grid",
        "available=off\navailable_3_2=off\navailable_3_2=off",
    ] {
        assert!(
            registry().validate_catalog(text).is_err(),
            "accepted {text:?}"
        );
    }
}

#[test]
fn catalog_errors_identify_the_file_and_line() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("gba.ini"),
        "available=off\navailable_3_2=missing\n",
    )
    .unwrap();
    let error = registry().validate_catalogs(dir.path()).unwrap_err();
    assert!(error.contains("gba.ini: line 2: unknown shader"), "{error}");
}
