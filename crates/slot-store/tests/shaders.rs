use slot_store::{shader_catalog, Platform, Shader, ShaderCatalog};

#[test]
fn bundled_catalogs_preserve_the_existing_order() {
    assert_eq!(shader_catalog(Platform::Gba).available(), Shader::GBA);
    assert_eq!(shader_catalog(Platform::Gb).available(), Shader::GB);
    assert_eq!(shader_catalog(Platform::Gbc).available(), Shader::GB);
}

#[test]
fn only_gba_has_a_three_by_two_override() {
    for panel in [(720, 480), (1440, 960)] {
        assert_eq!(
            shader_catalog(Platform::Gba).choices(panel),
            Shader::GBA_3_2
        );
        assert_eq!(shader_catalog(Platform::Gb).choices(panel), Shader::GB);
        assert_eq!(shader_catalog(Platform::Gbc).choices(panel), Shader::GB);
    }
    for panel in [(640, 480), (720, 720), (0, 0), (0, 480)] {
        assert_eq!(shader_catalog(Platform::Gba).choices(panel), Shader::GBA);
    }
}

#[test]
fn a_panel_override_does_not_invalidate_a_saved_selection() {
    let catalog = shader_catalog(Platform::Gba);
    assert!(catalog.supports(Shader::Shimmerless));
    assert!(!catalog.choices((720, 480)).contains(&Shader::Shimmerless));
    assert!(!catalog.supports(Shader::Simpletex));
    assert!(!shader_catalog(Platform::Gb).supports(Shader::Dot));
}

#[test]
fn gba_video_shares_the_cached_gba_catalog() {
    assert!(std::ptr::eq(
        shader_catalog(Platform::Gba),
        shader_catalog(Platform::Gbav)
    ));
    assert!(std::ptr::eq(
        shader_catalog(Platform::Gb),
        shader_catalog(Platform::Gb)
    ));
}

#[test]
fn configured_order_controls_forward_and_backward_cycling() {
    let catalog = ShaderCatalog::parse("available=dot,off,grid\navailable_3_2=grid,dot\n").unwrap();
    assert_eq!(
        catalog.available(),
        [Shader::Dot, Shader::Off, Shader::Grid]
    );
    assert_eq!(
        Shader::Off.step(catalog.choices((640, 480)), true),
        Shader::Grid
    );
    assert_eq!(
        Shader::Dot.step(catalog.choices((640, 480)), false),
        Shader::Grid
    );
    assert_eq!(catalog.choices((720, 480)), [Shader::Grid, Shader::Dot]);
}

#[test]
fn comments_and_whitespace_are_accepted() {
    let catalog =
        ShaderCatalog::parse("\n# comment\n; comment\n available = off , grid \n").unwrap();
    assert_eq!(catalog.available(), [Shader::Off, Shader::Grid]);
}

#[test]
fn unknown_shader_names_are_rejected_in_either_list() {
    for text in [
        "available=off,unknown",
        "available=off\navailable_3_2=unknown",
    ] {
        assert!(ShaderCatalog::parse(text)
            .unwrap_err()
            .contains("unknown shader"));
    }
}

#[test]
fn duplicate_shader_names_are_rejected_in_either_list() {
    for text in ["available=off,off", "available=off\navailable_3_2=off,off"] {
        assert!(ShaderCatalog::parse(text)
            .unwrap_err()
            .contains("duplicate shader"));
    }
}

#[test]
fn missing_empty_or_partially_empty_lists_are_rejected() {
    for text in [
        "",
        "available_3_2=off",
        "available=",
        "available=off,",
        "available=off\navailable_3_2=",
    ] {
        assert!(ShaderCatalog::parse(text).is_err(), "accepted {text:?}");
    }
}

#[test]
fn panel_override_must_be_a_subset_of_available_shaders() {
    assert!(ShaderCatalog::parse("available=off\navailable_3_2=grid")
        .unwrap_err()
        .contains("unavailable shader"));
}

#[test]
fn malformed_unknown_and_duplicate_keys_are_rejected() {
    for text in [
        "available off",
        "availble=off",
        "available=off\navailable=grid",
        "available=off\navailable_3_2=off\navailable_3_2=off",
    ] {
        assert!(ShaderCatalog::parse(text).is_err(), "accepted {text:?}");
    }
}

#[test]
fn aspect_ratio_comparison_handles_large_dimensions() {
    assert_eq!(
        shader_catalog(Platform::Gba).choices((u32::MAX, u32::MAX)),
        Shader::GBA
    );
}

#[cfg(feature = "nds")]
#[test]
fn ds_catalog_only_offers_off() {
    assert_eq!(shader_catalog(Platform::Nds).available(), [Shader::Off]);
    assert_eq!(
        shader_catalog(Platform::Nds).choices((720, 480)),
        [Shader::Off]
    );
}
