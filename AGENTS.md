# Repository Guidelines

## Project Structure & Module Organization

This Rust 2021 workspace targets macOS development and ARM64 Linux handhelds. `crates/slot` contains the executable, application state, sessions, and emulation coordination. Supporting crates separate UI (`slot-ui`), rendering (`slot-gfx`), emulator integration (`slot-retro`), storage (`slot-store`), input (`slot-input`), and power management (`slot-power`). `slot-sfxcut` prepares sound effects.

Implementation lives in each crate’s `src/`; integration tests live in `tests/`. Artwork resides in `assets/` and crate-local asset directories. `cores/` holds emulator patches and build scripts; `card/` holds device scripts and distribution files. `site/` contains the Zensical documentation website.

## Build, Test, and Development Commands

Use the pinned Rust 1.96 toolchain. Install Task and cargo-nextest; host core builds also require CMake and native compiler tools. Run commands from the repository root:

- `cargo build --locked --workspace`: build all packages.
- `cargo build --locked --release -p slot`: build the optimized application.
- `task core`: build patched mGBA and fetch the macOS gpSP core.
- `task sdcard`: initialize `./sdcard`; add unzipped ROMs under `Games/GBA`, `Games/GB`, or `Games/GBC`.
- `task run`: obtain cores and launch locally; override content with `SLOT_ROOT`.
- `task check`: check formatting, lint host/device features, and run Rust and card-script tests.
- `task build:device`: build ARM64 Linux output, using Docker when needed.

## Coding Style & Naming Conventions

Follow rustfmt’s four-space indentation and existing module organization. Use `snake_case` for modules, functions, and tests; `PascalCase` for types; and `SCREAMING_SNAKE_CASE` for constants. Run `cargo fmt --all` before submission. Clippy checks use `-D warnings`; check both host and device configurations. Keep platform-specific changes compatible with the relevant Cargo features.

## Testing Guidelines

Use Rust `#[test]` tests and descriptive behavior-based names. Add regression coverage for changed behavior in the owning crate. Run focused tests with commands such as `cargo test -p slot-store --test scan`. Preserve `task test`’s nextest/Cargo split. Run `task test:card` for shell changes. Real-core tests may skip without cores or ROM/BIOS fixtures; report skips. No numeric coverage threshold is specified.

## Commit & Pull Request Guidelines

Recent commits use concise imperative subjects, sometimes ending with `closes #140`; Conventional Commit prefixes are not customary. Describe the problem, resulting behavior, relevant issues, and validation in PRs. Include screenshots for visible UI changes and note platform-specific limitations or skipped checks.
