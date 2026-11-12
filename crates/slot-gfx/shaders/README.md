# Built-in shaders

## Source layout and assembly

`../src/shaders.rs` embeds these GLSL files with `include_str!`. The
`GAME_FRAG` constant uses `concat!` to join `game.frag.glsl` and the seven files
under `effects/` into one fragment shader. Each effect file owns its function
and helpers; `game.frag.glsl` owns shared uniforms, sampling, clipping, mode
dispatch, and brightness. Off leaves the sampled colour unchanged.

`GamePass::new` in `../src/pipeline.rs` compiles this source with
`game.vert.glsl`. Switching effects updates `u_mode` without recompiling.
`ScreenEffect::mode` in `../src/lcd3x.rs` supplies the numeric mode; preserve
its correspondence with the dispatch thresholds when editing effects.

The other files implement sprite and paper drawing, final blitting, panel
rotation, and frame blending. Keep those programs separate from game effects.

`shaders::program` supplies compatibility preambles for desktop OpenGL 3.3
and OpenGL ES 2.0. Keep effect sources compatible with both backends and
preserve precision qualifiers. Sources are embedded at build time; no shader
files need to be installed beside the executable.

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
GBA with Dark Gritty Retro excluded. GB and GBC have separate catalogs and
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
task check
cargo test -p slot-gfx --test pipeline
cargo test -p slot --test render_picture_modes --test render_layout_golden
cargo build --locked --release -p slot
task build:device
```

Rendering tests require working OpenGL access. Review unexpected image
changes before updating any golden fixtures. Desktop tests do not verify
handheld OpenGL ES rendering: also check each available effect, switching,
saved settings after restart, and panel-specific behavior on hardware.

Shader origins, authors, licenses, and port modifications are recorded in
[`licenses/shaders.txt`](../../../licenses/shaders.txt). Preserve this attribution
when changing or moving sources.
