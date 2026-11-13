# Built-in shaders

## Source layout and assembly

`registry.toml` registers selectable shaders separately from rendering effects.
`slot-shader-build`, used only by build scripts, generates graphics, saved
identities, application mappings, menu values, and toasts into Cargo's `OUT_DIR`.
Generated Rust retains the existing enums and direct matches; there is no
runtime registry lookup or discovery.

`../src/shaders.rs` embeds the generated fragment shader with `include_str!`.
Generation fills the declaration and dispatch markers in `game.frag.glsl`,
then appends each registered effect source in registry order. Each effect file
owns its function and helpers; the fragment template owns shared uniforms,
sampling, clipping, and brightness. Off leaves the sampled colour unchanged.

`GamePass::new` in `../src/pipeline.rs` compiles this source with
`game.vert.glsl`. Switching effects updates `u_mode` without recompiling.
`ScreenEffect::mode` in `../src/lcd3x.rs` supplies the explicit registry mode.
The generator derives dispatch thresholds from those same mode numbers.

The other files implement sprite and paper drawing, final blitting, panel
rotation, and frame blending. Keep those programs separate from game effects.

`shaders::program` supplies compatibility preambles for desktop OpenGL 3.3
and OpenGL ES 2.0. Keep effect sources compatible with both backends and
preserve precision qualifiers. Sources are embedded at build time; no shader
files need to be installed beside the executable.

## Adding a shader

An ordinary single-pass effect needs three file changes:

1. Create `effects/soft.glsl`, with a uniquely named entry point and helpers:

   ```glsl
   void effect_soft(inout vec3 c, vec2 uv, vec2 f) {
       c *= 0.95;
   }
   ```

2. Register its implementation and selectable identity in `registry.toml`:

   ```toml
   [[effects]]
   id = "Soft"
   mode = 8
   source = "effects/soft.glsl"
   entry = "effect_soft"

   [[shaders]]
   id = "Soft"
   name = "soft"
   label = "Soft"
   gba_video = true
   mapping = { kind = "fixed", effect = "Soft" }
   ```

3. Add `soft` at the intended cycling position in the relevant system INI's
   `available` list. Include it in `available_3_2` too if that system's override
   should offer it. Enabling more systems requires editing their INIs as well.

Keep existing IDs, saved names, and modes stable. IDs become Rust variants;
saved names are lowercase identifiers with optional hyphens. Choose a unique
nonzero mode that fits in a `u8`. Append new effects to preserve the existing
source assembly order, and append selections to preserve existing UI order.
Use names that do not collide with non-shader variants in the UI templates.
Labels supply both menu text and the `Shader: ...` toast text; check long labels
on the narrowest supported panel. Off shares the ordinary menu's Off value.

LCD demonstrates a panel-dependent mapping: one selection chooses between two
rendering implementations. Effects requiring new textures, uniforms, geometry,
or passes still need rendering changes; registration does not provide those
resources. Grid's special source geometry remains in `GamePass`.

Do not edit generated files. Non-shader UI definitions remain in
`../../slot-ui/templates/{quick_values,toasts}.rs.in`. Cargo tracks the registry,
GLSL sources, fragment/UI templates, and system catalog directory for rebuilds.

Build-time diagnostics reject duplicate IDs, names, modes, sources, and entry
points; invalid or unused effect mappings; missing/unreadable GLSL sources;
invalid Off/LCD registrations; unknown INI names; and missing/duplicate template
markers. File reads, catalog errors, and template errors identify the input
path. GLSL syntax and entry-point signatures are checked when the graphics
program compiles, so run the GPU tests too.

The three-file workflow covers registration. Add effect-specific reference
tests and attribution when appropriate; update compatibility assertions if the
new choice deliberately changes list sizes or the order of later UI values.

## System availability

The ordered availability lists live in
`../../slot-store/config/shaders/gba.ini`, `gb.ini`, and `gbc.ini`:

```ini
available=off,lcd3x,grid,dot,shimmerless,dark-gritty-retro
available_3_2=off,lcd3x,grid,dot,dark-gritty-retro
```

`available` controls supported saved selections and normal cycling order.
The optional `available_3_2` overrides the choices on 3:2 panels and must be a
subset of `available`. Lists must be nonempty and contain unique, known
`Shader::name()` identifiers. Unknown keys, duplicate keys, and malformed
entries are rejected. Blank lines and full-line `#` or `;` comments are accepted.

`slot-store/src/shaders.rs` embeds and validates catalogs once per process.
Changes require rebuilding. These files are repository configuration, not
SD-card overrides or custom shader presets. GBA Video derives its catalog from
GBA using each selection's `gba_video` flag (currently excluding Dark Gritty
Retro). GB and GBC have separate catalogs and
saved selections. NDS offers only Off.

Defaults remain in `SlotState::default`. GBA starts with Grid; GB and GBC
start with Simpletex. LCD resolves to LCD3x on 3:2 panels and LCD Grid on
other panels. A saved GBA Shimmerless selection remains saved when a 3:2
panel displays it as Off, so switching back restores it.

## Dark Gritty Retro

`effects/dark_gritty_retro.glsl` ports the supplied Hyllian/DOLLS Retro shader
as a fixed built-in effect. Its preset uses one nearest-sampled pass at the
picture's output size, without extra textures or runtime preset loading.
The port preserves selective color grading, vignette, pixel edges, and the
original unequal input/output gamma exponents. Helpers and constants are
namespaced; source/output pixel ratios use the sampled UV span and actual
framebuffer geometry, including cropping.

The menu calls it Dark Gritty Retro; saved settings use `dark-gritty-retro`.
It is available for GBA games on both panel shapes. GBA Video displays Off
for this shared saved selection without erasing it; returning to GBA restores
it. Existing brightness, color correction, frame blending, and final display
adjustments still compose with this effect. Defaults remain unchanged.

The original source is retained in `../tests/fixtures/dark-gritty-retro.glsl`
for independent GPU comparisons. Attribution and GPL notices are preserved.

## Verification and attribution

Run from the repository root:

```sh
cargo test --locked -p slot-shader-build
task check
cargo test -p slot-gfx --test pipeline
cargo test -p slot --test render_picture_modes --test render_layout_golden
cargo build --locked --release -p slot
task build:device
```

Rendering tests require working OpenGL access. Review unexpected image
changes before updating any golden fixtures. Desktop tests do not verify
handheld OpenGL ES rendering: also check each available effect, switching,
saved settings after restart, panel-specific behavior, and frame pacing on
hardware. Adding an expensive effect can increase GPU work even though
registration itself has no runtime overhead.

Shader origins, authors, licenses, and port modifications are recorded in
[`licenses/shaders.txt`](../../../licenses/shaders.txt). Preserve this attribution
when changing or moving sources.
