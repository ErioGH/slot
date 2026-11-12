use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use slot_shader_build::{generate_app, generate_gfx, generate_store, generate_ui, Registry};

fn shader_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slot-gfx/shaders")
}

fn config_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slot-store/config/shaders")
}

fn template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slot-ui/templates")
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("effects")).unwrap();
    for path in ["registry.toml", "game.frag.glsl"] {
        fs::copy(shader_dir().join(path), dir.path().join(path)).unwrap();
    }
    let registry = Registry::load(&shader_dir().join("registry.toml")).unwrap();
    for effect in &registry.effects {
        if let Some(source) = &effect.source {
            fs::copy(shader_dir().join(source), dir.path().join(source)).unwrap();
        }
    }
    dir
}

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name)).unwrap()
}

#[test]
fn a_shader_integrates_with_only_a_source_registry_and_system_config_edit() {
    let dir = fixture();
    let config = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    for name in ["gba.ini", "gb.ini", "gbc.ini"] {
        fs::copy(config_dir().join(name), config.path().join(name)).unwrap();
    }

    // These are the only registration inputs changed in the isolated fixture.
    let source = "void effect_synthetic(inout vec3 c, vec2 uv, vec2 f) { c *= 0.5; }\n";
    fs::write(dir.path().join("effects/synthetic.glsl"), source).unwrap();
    let mut registry = read(dir.path(), "registry.toml");
    registry.push_str(
        r#"
[[effects]]
id = "Synthetic"
mode = 8
source = "effects/synthetic.glsl"
entry = "effect_synthetic"

[[shaders]]
id = "Synthetic"
name = "synthetic"
label = "Synthetic"
gba_video = false
mapping = { kind = "fixed", effect = "Synthetic" }
"#,
    );
    fs::write(dir.path().join("registry.toml"), registry).unwrap();
    let catalog =
        read(config.path(), "gba.ini").replace("dark-gritty-retro", "dark-gritty-retro,synthetic");
    fs::write(config.path().join("gba.ini"), catalog).unwrap();

    generate_gfx(dir.path(), out.path()).unwrap();
    generate_store(dir.path(), config.path(), out.path()).unwrap();
    generate_app(dir.path(), out.path()).unwrap();
    generate_ui(dir.path(), &template_dir(), out.path()).unwrap();
    let frag = read(out.path(), "game.frag.glsl");
    assert!(frag.contains("void effect_synthetic(inout vec3 c, vec2 uv, vec2 f);"));
    assert!(frag.contains("} else if (u_mode > 7.5) {\n        effect_synthetic(c, uv, f);"));
    assert!(frag.ends_with(source));

    // Compile all generated Rust against minimal stand-ins for the unrelated
    // application/UI types. No manually maintained shader registration is added.
    let harness = r#"
mod slot_gfx {
    include!(concat!(env!("OUT_DIR"), "/screen_effect.rs"));
}
include!(concat!(env!("OUT_DIR"), "/shader.rs"));
include!(concat!(env!("OUT_DIR"), "/quick_values.rs"));
include!(concat!(env!("OUT_DIR"), "/toasts.rs"));
include!(concat!(env!("OUT_DIR"), "/shader_app.rs"));

struct App { shader: Shader, panel: (u32, u32) }
impl App { fn screen_shader(&self) -> Shader { self.shader } }
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GbColors;
impl GbColors {
    fn all() -> impl Iterator<Item = Self> { [Self].into_iter() }
    fn index(self) -> usize { 0 }
    fn label(self) -> &'static str { "Auto" }
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct QuickRow;
impl QuickRow {
    const ALL: [Self; 1] = [Self];
    fn binds(self) -> bool { true }
    fn game_button(self) -> Option<()> { None }
    fn label(self) -> &'static str { "Stub" }
}
pub enum Icon { Alert, Camera }
mod hud { pub type Millis = u64; pub const HUD_MS: Millis = 1500; }
const REPAIR_HOLD_MS: hud::Millis = 6000;
const WARNING_HOLD_MS: hud::Millis = 4000;
const TINT_HOLD_MS: hud::Millis = 1000;
fn bound_rows() -> impl Iterator<Item = QuickRow> { QuickRow::ALL.into_iter() }

#[test]
fn synthetic_shader_is_registered_everywhere() {
    assert_eq!(Shader::Synthetic.name(), "synthetic");
    assert_eq!(Shader::parse("synthetic"), Some(Shader::Synthetic));
    assert!(!Shader::Synthetic.supports_gba_video());
    let app = App { shader: Shader::Synthetic, panel: (640, 480) };
    assert_eq!(app.screen_effect(), slot_gfx::ScreenEffect::Synthetic);
    assert_eq!(slot_gfx::ScreenEffect::Synthetic as u8, 8);
    assert_eq!(shader_value(Shader::Synthetic), QuickValue::Synthetic);
    assert_eq!(shader_toast(Shader::Synthetic), Toast::ShaderSynthetic);
    assert_eq!(QuickValue::Synthetic.text(), "Synthetic");
    assert_eq!(Toast::ShaderSynthetic.text(), "Shader: Synthetic");
    assert_eq!(QuickValue::ALL.len(), 59);
    assert_eq!(Toast::FIXED.len(), 34);
    for (index, value) in QuickValue::ALL.into_iter().enumerate() {
        assert_eq!(value.index(), index);
    }
    for (index, toast) in Toast::all().into_iter().enumerate() {
        assert_eq!(toast.index(), index);
    }
    assert_eq!(QuickValue::ALL.into_iter().filter(|v| *v == QuickValue::Off).count(), 1);
}
"#;
    let path = out.path().join("integration.rs");
    fs::write(&path, harness).unwrap();
    let executable = out
        .path()
        .join(format!("integration{}", std::env::consts::EXE_SUFFIX));
    let result = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .args(["--edition=2021", "--test"])
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .env("OUT_DIR", out.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = Command::new(executable).output().unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn generated_fragment_preserves_existing_glsl_except_prototype_order() {
    let out = tempfile::tempdir().unwrap();
    generate_gfx(&shader_dir(), out.path()).unwrap();
    let actual = read(out.path(), "game.frag.glsl");
    let expected = concat!(
        // The fragment before generation was introduced. Source bodies remain
        // separate so their effect-specific tests continue to own those changes.
        r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D u_game;
uniform sampler2D u_paper;
uniform vec2 u_src;
uniform vec4 u_uv;
uniform float u_bright;
uniform float u_mode;
uniform float u_paper_size;
uniform vec2 u_origin;
uniform vec4 u_rect;
uniform vec2 u_fbo;
uniform vec4 u_pic;
varying vec2 v_uv;
void effect_lcd_grid(inout vec3 c, vec2 uv, vec2 f);
void effect_shimmerless(inout vec3 c, vec2 uv, vec2 f);
void effect_simpletex(inout vec3 c, vec2 uv, vec2 f);
void effect_dot(inout vec3 c, vec2 uv, vec2 f);
void effect_grid(inout vec3 c, vec2 uv, vec2 f);
void effect_lcd3x(inout vec3 c, vec2 uv, vec2 f);
void effect_dark_gritty_retro(inout vec3 c, vec2 uv, vec2 f);
void main() {
    vec2 uv = u_uv.xy + v_uv * u_uv.zw;
    vec2 f = fract(uv * u_src);
    vec3 c = texture2D(u_game, uv).rgb;
    bool inside = all(greaterThanEqual(uv, u_pic.xy)) && all(lessThan(uv, u_pic.zw));
    if (!inside) {
    } else if (u_mode > 6.5) {
        effect_dark_gritty_retro(c, uv, f);
    } else if (u_mode > 5.5) {
        effect_lcd_grid(c, uv, f);
    } else if (u_mode > 4.5) {
        effect_shimmerless(c, uv, f);
    } else if (u_mode > 3.5) {
        effect_simpletex(c, uv, f);
    } else if (u_mode > 2.5) {
        effect_dot(c, uv, f);
    } else if (u_mode > 1.5) {
        effect_grid(c, uv, f);
    } else if (u_mode > 0.5) {
        effect_lcd3x(c, uv, f);
    }
    FRAG_COLOR = vec4(c * u_bright, 1.0);
}
"#,
        include_str!("../../slot-gfx/shaders/effects/dark_gritty_retro.glsl"),
        include_str!("../../slot-gfx/shaders/effects/lcd_grid.glsl"),
        include_str!("../../slot-gfx/shaders/effects/shimmerless.glsl"),
        include_str!("../../slot-gfx/shaders/effects/simpletex.glsl"),
        include_str!("../../slot-gfx/shaders/effects/dot.glsl"),
        include_str!("../../slot-gfx/shaders/effects/grid.glsl"),
        include_str!("../../slot-gfx/shaders/effects/lcd3x.glsl"),
    );
    let split = |text: &str| {
        let (mut prototypes, body): (Vec<_>, Vec<_>) = text
            .lines()
            .partition(|line| line.starts_with("void effect_") && line.ends_with(';'));
        prototypes.sort();
        (prototypes.join("\n"), body.join("\n"))
    };
    assert_eq!(split(&actual), split(expected));
    assert!(!actual.contains("@effect_"));
}

#[test]
fn generated_enum_keeps_existing_order_traits_and_explicit_modes() {
    let out = tempfile::tempdir().unwrap();
    generate_gfx(&shader_dir(), out.path()).unwrap();
    assert_eq!(
        read(out.path(), "screen_effect.rs"),
        "// Generated from shaders/registry.toml.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ScreenEffect {
    None = 0,
    Lcd3x = 1,
    Grid = 2,
    Dot = 3,
    Simpletex = 4,
    Shimmerless = 5,
    LcdGrid = 6,
    DarkGrittyRetro = 7,
}
"
    );
}

#[test]
fn output_is_deterministic_and_independent_of_output_directory() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let inputs = generate_gfx(&shader_dir(), first.path()).unwrap();
    assert_eq!(inputs, generate_gfx(&shader_dir(), second.path()).unwrap());
    let inputs = generate_store(&shader_dir(), &config_dir(), first.path()).unwrap();
    assert_eq!(
        inputs,
        generate_store(&shader_dir(), &config_dir(), second.path()).unwrap()
    );
    let inputs = generate_app(&shader_dir(), first.path()).unwrap();
    assert_eq!(inputs, generate_app(&shader_dir(), second.path()).unwrap());
    let inputs = generate_ui(&shader_dir(), &template_dir(), first.path()).unwrap();
    assert_eq!(
        inputs,
        generate_ui(&shader_dir(), &template_dir(), second.path()).unwrap()
    );
    for name in [
        "screen_effect.rs",
        "game.frag.glsl",
        "shader.rs",
        "shader_app.rs",
        "quick_values.rs",
        "toasts.rs",
    ] {
        assert_eq!(read(first.path(), name), read(second.path(), name));
    }
    generate_gfx(&shader_dir(), first.path()).unwrap();
    generate_store(&shader_dir(), &config_dir(), first.path()).unwrap();
    generate_app(&shader_dir(), first.path()).unwrap();
    generate_ui(&shader_dir(), &template_dir(), first.path()).unwrap();
    for name in [
        "screen_effect.rs",
        "game.frag.glsl",
        "shader.rs",
        "shader_app.rs",
        "quick_values.rs",
        "toasts.rs",
    ] {
        assert_eq!(read(first.path(), name), read(second.path(), name));
    }
}

#[test]
fn all_registry_template_and_source_inputs_are_tracked() {
    let out = tempfile::tempdir().unwrap();
    let inputs = generate_gfx(&shader_dir(), out.path()).unwrap();
    let expected = [
        "registry.toml",
        "game.frag.glsl",
        "effects/dark_gritty_retro.glsl",
        "effects/lcd_grid.glsl",
        "effects/shimmerless.glsl",
        "effects/simpletex.glsl",
        "effects/dot.glsl",
        "effects/grid.glsl",
        "effects/lcd3x.glsl",
    ]
    .map(|path| shader_dir().join(path));
    assert_eq!(inputs, expected);

    let app_inputs = generate_app(&shader_dir(), out.path()).unwrap();
    assert_eq!(app_inputs, [&expected[..1], &expected[2..]].concat());
    let store_inputs = generate_store(&shader_dir(), &config_dir(), out.path()).unwrap();
    assert_eq!(
        store_inputs,
        [&expected[..1], &[config_dir()], &expected[2..]].concat()
    );
    let ui_inputs = generate_ui(&shader_dir(), &template_dir(), out.path()).unwrap();
    let templates = [
        template_dir().join("quick_values.rs.in"),
        template_dir().join("toasts.rs.in"),
    ];
    assert_eq!(
        ui_inputs,
        [&expected[..1], &templates, &expected[2..]].concat()
    );
}

#[test]
fn ui_generation_registers_each_shader_and_shares_the_off_menu_value() {
    let out = tempfile::tempdir().unwrap();
    generate_ui(&shader_dir(), &template_dir(), out.path()).unwrap();
    let quick = read(out.path(), "quick_values.rs");
    let toast = read(out.path(), "toasts.rs");
    for (id, label) in [
        ("Off", "Off"),
        ("Lcd3x", "LCD"),
        ("Grid", "Grid"),
        ("Dot", "Dot"),
        ("Simpletex", "Simpletex"),
        ("Shimmerless", "Shimmerless"),
        ("DarkGrittyRetro", "Dark Gritty Retro"),
    ] {
        assert_eq!(quick.matches(&format!("\n    {id},")).count(), 1);
        assert_eq!(
            quick
                .matches(&format!("\n        QuickValue::{id},"))
                .count(),
            1
        );
        assert!(quick.contains(&format!("QuickValue::{id} => {label:?},")));
        assert_eq!(toast.matches(&format!("\n    Shader{id},")).count(), 1);
        assert_eq!(
            toast
                .matches(&format!("\n        Toast::Shader{id},"))
                .count(),
            1
        );
        assert!(toast.contains(&format!(
            "Toast::Shader{id} => {:?},",
            format!("Shader: {label}")
        )));
    }
    assert!(quick.contains("pub const ALL: [QuickValue; 52 + 6]"));
    assert!(toast.contains("const FIXED: [Toast; 26 + 7]"));
    assert!(quick.contains("self as usize"));
    assert!(!quick.contains('@'));
    assert!(!toast.contains('@'));
}

#[test]
fn ui_generation_uses_registry_labels_and_escapes_rust_strings() {
    let dir = fixture();
    let out = tempfile::tempdir().unwrap();
    let text = read(dir.path(), "registry.toml").replace(
        "label = \"Dark Gritty Retro\"",
        r#"label = 'Dark "Gritty" \ Retro'"#,
    );
    fs::write(dir.path().join("registry.toml"), text).unwrap();
    generate_ui(dir.path(), &template_dir(), out.path()).unwrap();
    let label = "Dark \"Gritty\" \\ Retro";
    assert!(read(out.path(), "quick_values.rs")
        .contains(&format!("QuickValue::DarkGrittyRetro => {label:?},")));
    assert!(read(out.path(), "toasts.rs").contains(&format!(
        "Toast::ShaderDarkGrittyRetro => {:?},",
        format!("Shader: {label}")
    )));
}

#[test]
fn malformed_ui_templates_report_missing_or_duplicate_markers() {
    let templates = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    for (name, markers) in [
        (
            "quick_values.rs.in",
            [
                "    // @shader_variants@",
                "        // @shader_values@",
                "            // @shader_labels@",
                "@shader_value_count@",
            ],
        ),
        (
            "toasts.rs.in",
            [
                "    // @shader_variants@",
                "        // @shader_toasts@",
                "            // @shader_labels@",
                "@shader_toast_count@",
            ],
        ),
    ] {
        for marker in markers {
            for replacement in [String::new(), format!("{marker}\n{marker}")] {
                for file in ["quick_values.rs.in", "toasts.rs.in"] {
                    fs::copy(template_dir().join(file), templates.path().join(file)).unwrap();
                }
                let template = read(&template_dir(), name).replace(marker, &replacement);
                fs::write(templates.path().join(name), template).unwrap();
                let error = generate_ui(&shader_dir(), templates.path(), out.path()).unwrap_err();
                assert!(
                    error.contains(&format!("{name}: expected exactly one")),
                    "{error}"
                );
                assert!(error.contains(marker), "{error}");
                assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
            }
        }
    }
}

#[test]
fn storage_generation_uses_registered_saved_names_and_video_eligibility() {
    let dir = fixture();
    let config = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let text = read(dir.path(), "registry.toml")
        .replace("name = \"grid\"", "name = \"grid-new\"")
        .replace(
            "label = \"Grid\"\ngba_video = true",
            "label = \"Grid\"\ngba_video = false",
        );
    fs::write(dir.path().join("registry.toml"), text).unwrap();
    fs::write(config.path().join("gba.ini"), "available=off,grid-new").unwrap();
    generate_store(dir.path(), config.path(), out.path()).unwrap();
    let rust = read(out.path(), "shader.rs");
    assert!(rust.contains("Shader::Grid => \"grid-new\","));
    assert!(rust.contains("\"grid-new\" => Some(Shader::Grid),"));
    assert!(rust.contains("Shader::Grid => false,"));
    assert!(!rust.contains("\"grid\""));
}

#[test]
fn storage_generation_rejects_unknown_system_config_names() {
    let config = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    fs::write(config.path().join("gba.ini"), "available=off,unknown").unwrap();
    let error = generate_store(&shader_dir(), config.path(), out.path()).unwrap_err();
    assert!(error.contains("gba.ini: line 1: unknown shader"), "{error}");
    assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
}

#[test]
fn application_generation_covers_effect_menu_and_toast_mappings() {
    let out = tempfile::tempdir().unwrap();
    generate_app(&shader_dir(), out.path()).unwrap();
    let rust = read(out.path(), "shader_app.rs");
    for (id, effect) in [
        ("Off", "None"),
        ("Grid", "Grid"),
        ("Dot", "Dot"),
        ("Simpletex", "Simpletex"),
        ("Shimmerless", "Shimmerless"),
        ("DarkGrittyRetro", "DarkGrittyRetro"),
    ] {
        assert!(rust.contains(&format!("Shader::{id} => ScreenEffect::{effect},")));
    }
    for id in [
        "Off",
        "Lcd3x",
        "Grid",
        "Dot",
        "Simpletex",
        "Shimmerless",
        "DarkGrittyRetro",
    ] {
        assert!(rust.contains(&format!("Shader::{id} => QuickValue::{id},")));
        assert!(rust.contains(&format!("Shader::{id} => Toast::Shader{id},")));
    }
    assert!(rust.contains("true => ScreenEffect::Lcd3x,"));
    assert!(rust.contains("false => ScreenEffect::LcdGrid,"));
}

#[test]
fn application_generation_uses_registered_lcd_targets() {
    let dir = fixture();
    let out = tempfile::tempdir().unwrap();
    let text = read(dir.path(), "registry.toml").replace(
        "panel_3_2 = \"Lcd3x\", other = \"LcdGrid\"",
        "panel_3_2 = \"LcdGrid\", other = \"Lcd3x\"",
    );
    fs::write(dir.path().join("registry.toml"), text).unwrap();
    generate_app(dir.path(), out.path()).unwrap();
    let rust = read(out.path(), "shader_app.rs");
    assert!(rust.contains("true => ScreenEffect::LcdGrid,"));
    assert!(rust.contains("false => ScreenEffect::Lcd3x,"));
}

#[test]
fn explicit_registry_modes_control_discriminants_and_dispatch_thresholds() {
    let dir = fixture();
    let out = tempfile::tempdir().unwrap();
    let text = read(dir.path(), "registry.toml").replace("mode = 7", "mode = 9");
    fs::write(dir.path().join("registry.toml"), text).unwrap();
    generate_gfx(dir.path(), out.path()).unwrap();
    let rust = read(out.path(), "screen_effect.rs");
    let frag = read(out.path(), "game.frag.glsl");
    assert!(rust.contains("DarkGrittyRetro = 9,"));
    assert!(
        frag.contains("} else if (u_mode > 8.5) {\n        effect_dark_gritty_retro(c, uv, f);")
    );
    assert!(frag.find("u_mode > 8.5").unwrap() < frag.find("u_mode > 5.5").unwrap());
    assert!(!frag.contains("u_mode > 6.5"));
}

#[test]
fn malformed_templates_report_missing_or_duplicate_markers() {
    let dir = fixture();
    let out = tempfile::tempdir().unwrap();
    let template = read(dir.path(), "game.frag.glsl");
    for marker in ["// @effect_declarations@", "    // @effect_dispatch@"] {
        for replacement in [String::new(), format!("{marker}\n{marker}")] {
            fs::write(
                dir.path().join("game.frag.glsl"),
                template.replace(marker, &replacement),
            )
            .unwrap();
            let error = generate_gfx(dir.path(), out.path()).unwrap_err();
            assert!(
                error.contains("game.frag.glsl: expected exactly one"),
                "{error}"
            );
            assert!(error.contains(marker), "{error}");
        }
    }
}

#[test]
fn invalid_registry_fails_before_writing_outputs() {
    let dir = fixture();
    let out = tempfile::tempdir().unwrap();
    let text = read(dir.path(), "registry.toml").replace("mode = 7", "mode = 6");
    fs::write(dir.path().join("registry.toml"), text).unwrap();
    let error = generate_gfx(dir.path(), out.path()).unwrap_err();
    assert!(error.contains("duplicate GPU mode 6"), "{error}");
    let error = generate_store(dir.path(), &config_dir(), out.path()).unwrap_err();
    assert!(error.contains("duplicate GPU mode 6"), "{error}");
    let error = generate_app(dir.path(), out.path()).unwrap_err();
    assert!(error.contains("duplicate GPU mode 6"), "{error}");
    let error = generate_ui(dir.path(), &template_dir(), out.path()).unwrap_err();
    assert!(error.contains("duplicate GPU mode 6"), "{error}");
    assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
}
