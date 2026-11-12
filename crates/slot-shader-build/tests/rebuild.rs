use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use slot_shader_build::Registry;

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let helper = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shaders = helper.join("../slot-gfx/shaders");
    fs::create_dir_all(root.join("shaders/effects")).unwrap();
    fs::create_dir(root.join("templates")).unwrap();
    fs::create_dir(root.join("config")).unwrap();
    fs::create_dir(root.join("src")).unwrap();
    for name in ["registry.toml", "game.frag.glsl"] {
        fs::copy(shaders.join(name), root.join("shaders").join(name)).unwrap();
    }
    for effect in Registry::load(&shaders.join("registry.toml"))
        .unwrap()
        .effects
    {
        if let Some(source) = effect.source {
            fs::copy(shaders.join(&source), root.join("shaders").join(source)).unwrap();
        }
    }
    for name in ["quick_values.rs.in", "toasts.rs.in"] {
        fs::copy(
            helper.join("../slot-ui/templates").join(name),
            root.join("templates").join(name),
        )
        .unwrap();
    }
    for name in ["gba.ini", "gb.ini", "gbc.ini"] {
        fs::copy(
            helper.join("../slot-store/config/shaders").join(name),
            root.join("config").join(name),
        )
        .unwrap();
    }
    fs::write(root.join("Cargo.toml"), format!(
        "[package]\nname = \"shader-rebuild-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n\n[build-dependencies]\nslot-shader-build = {{ path = {:?} }}\n",
        helper.to_str().unwrap(),
    )).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "include!(concat!(env!(\"OUT_DIR\"), \"/screen_effect.rs\"));\n",
    )
    .unwrap();
    // Exercise Cargo's dependency tracking with the same returned inputs used
    // by the owning build scripts, without rebuilding unrelated runtime crates.
    fs::write(root.join("build.rs"), r#"
use std::{env, fs, path::PathBuf};
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let shaders = root.join("shaders");
    let mut inputs = slot_shader_build::generate_gfx(&shaders, &out).unwrap();
    inputs.extend(slot_shader_build::generate_store(&shaders, &root.join("config"), &out).unwrap());
    inputs.extend(slot_shader_build::generate_app(&shaders, &out).unwrap());
    inputs.extend(slot_shader_build::generate_ui(&shaders, &root.join("templates"), &out).unwrap());
    for input in inputs { println!("cargo:rerun-if-changed={}", input.display()); }
    let generation = out.join("generation");
    let previous: u32 = fs::read_to_string(&generation).unwrap_or_else(|_| "0".into()).parse().unwrap();
    fs::write(generation, (previous + 1).to_string()).unwrap();
}
"#).unwrap();
    dir
}

fn build(root: &Path) -> Output {
    // All helper dependencies were already fetched by the enclosing test build.
    // Use a separate target directory to avoid locking its build artifacts.
    Command::new(env!("CARGO"))
        .args(["build", "--offline", "--quiet"])
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .unwrap()
}

fn successful_build(root: &Path) {
    let result = build(root);
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}

fn append(root: &Path, path: &str, text: &str) {
    fs::write(root.join(path), read(root, path) + text).unwrap();
}

#[test]
fn cargo_regenerates_after_registry_sources_templates_and_catalogs_change() {
    let fixture = fixture();
    let root = fixture.path();
    successful_build(root);
    let out = fs::read_dir(root.join("target/debug/build"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join("out"))
        .find(|path| path.join("generation").exists())
        .unwrap();
    assert_eq!(read(&out, "generation"), "1");
    successful_build(root);
    assert_eq!(
        read(&out, "generation"),
        "1",
        "unchanged inputs must stay fresh"
    );

    let registry = read(root, "shaders/registry.toml")
        .replace("label = \"Grid\"", "label = \"Registry rebuilt\"");
    fs::write(root.join("shaders/registry.toml"), registry).unwrap();
    successful_build(root);
    assert_eq!(read(&out, "generation"), "2");
    assert!(read(&out, "quick_values.rs").contains("Registry rebuilt"));
    assert!(read(&out, "toasts.rs").contains("Shader: Registry rebuilt"));

    append(
        root,
        "shaders/effects/grid.glsl",
        "\n// effect source rebuilt\n",
    );
    successful_build(root);
    assert_eq!(read(&out, "generation"), "3");
    assert!(read(&out, "game.frag.glsl").contains("// effect source rebuilt"));

    append(
        root,
        "shaders/game.frag.glsl",
        "\n// fragment template rebuilt\n",
    );
    successful_build(root);
    assert_eq!(read(&out, "generation"), "4");
    assert!(read(&out, "game.frag.glsl").contains("// fragment template rebuilt"));

    append(
        root,
        "templates/quick_values.rs.in",
        "\n// quick values rebuilt\n",
    );
    successful_build(root);
    assert_eq!(read(&out, "generation"), "5");
    assert!(read(&out, "quick_values.rs").contains("// quick values rebuilt"));
    append(root, "templates/toasts.rs.in", "\n// toasts rebuilt\n");
    successful_build(root);
    assert_eq!(read(&out, "generation"), "6");
    assert!(read(&out, "toasts.rs").contains("// toasts rebuilt"));

    let catalog = read(root, "config/gba.ini").replace("off,lcd3x", "lcd3x,off");
    fs::write(root.join("config/gba.ini"), catalog).unwrap();
    successful_build(root);
    assert_eq!(
        read(&out, "generation"),
        "7",
        "catalog edits must rerun validation even if generated Rust is unchanged"
    );

    fs::write(root.join("config/new.ini"), "available=off\n").unwrap();
    successful_build(root);
    assert_eq!(
        read(&out, "generation"),
        "8",
        "new catalogs must be tracked"
    );
    fs::write(root.join("config/new.ini"), "available=unknown\n").unwrap();
    let result = build(root);
    assert!(
        !result.status.success(),
        "invalid catalog was accepted on rebuild"
    );
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("new.ini: line 1: unknown shader"), "{error}");
    fs::remove_file(root.join("config/new.ini")).unwrap();
    successful_build(root);
    assert_eq!(
        read(&out, "generation"),
        "9",
        "removed catalogs must be tracked"
    );
    successful_build(root);
    assert_eq!(read(&out, "generation"), "9");
}
