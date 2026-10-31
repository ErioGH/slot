#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};

use common::{clocked, core_lock, repo_root, tmp_root_with_gb_carts};
use slot::frontend::Frontend;
use slot_gfx::{Compositor, HeadlessSurface, OUT_H, OUT_W};
use slot_input::{Btn, InputSource, Millis, RawEvent};
use slot_power::SimPlatform;
use slot_store::{Core, Platform};

struct Script(VecDeque<Vec<RawEvent>>);

impl InputSource for Script {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn at(px: &[u8], x: usize, y: usize) -> [u8; 3] {
    let o = (y * OUT_W as usize + x) * 4;
    [px[o], px[o + 1], px[o + 2]]
}

const BAR_W: usize = 120;
const BAR_H: usize = 24;

fn lit(px: &[u8], xs: std::ops::Range<usize>, ys: std::ops::Range<usize>) -> usize {
    ys.flat_map(|y| xs.clone().map(move |x| (x, y)))
        .filter(|(x, y)| at(px, *x, *y).iter().any(|c| *c > 8))
        .count()
}

fn write_png(name: &str, w: u32, h: u32, rgba: &[u8]) {
    let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") else {
        return;
    };
    let path = format!("{dir}/{name}.png");
    let file = std::fs::File::create(&path).expect("create png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .expect("png header")
        .write_image_data(rgba)
        .expect("png data");
    println!("wrote {path}");
}

fn side_by_side(left: &[u8], right: &[u8], gap: usize) -> (u32, u32, Vec<u8>) {
    let w = OUT_W as usize * 2 + gap;
    let mut out = vec![0u8; w * OUT_H as usize * 4];
    for y in 0..OUT_H as usize {
        let row = y * w * 4;
        let src = y * OUT_W as usize * 4;
        let line = OUT_W as usize * 4;
        out[row..row + line].copy_from_slice(&left[src..src + line]);
        let over = row + line;
        for p in out[over..over + gap * 4].chunks_exact_mut(4) {
            p.copy_from_slice(&[0x60, 0x60, 0x60, 0xff]);
        }
        let from = over + gap * 4;
        out[from..from + line].copy_from_slice(&right[src..src + line]);
    }
    (w as u32, OUT_H, out)
}

fn zoom(
    left: &[u8],
    right: &[u8],
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    n: usize,
) -> (u32, u32, Vec<u8>) {
    let gap = 12;
    let out_w = w * n * 2 + gap;
    let out_h = h * n;
    let mut out = vec![0u8; out_w * out_h * 4];
    for y in 0..out_h {
        for x in 0..out_w {
            let (src, sx) = match x < w * n {
                true => (left, x / n),
                false if x < w * n + gap => {
                    let o = (y * out_w + x) * 4;
                    out[o..o + 4].copy_from_slice(&[0x60, 0x60, 0x60, 0xff]);
                    continue;
                }
                false => (right, (x - w * n - gap) / n),
            };
            let c = at(src, x0 + sx, y0 + y / n);
            let o = (y * out_w + x) * 4;
            out[o..o + 4].copy_from_slice(&[c[0], c[1], c[2], 0xff]);
        }
    }
    (out_w as u32, out_h as u32, out)
}

fn put_card_cart(root: &Path) -> Option<()> {
    let from = repo_root().join("sdcard/Games/GB/Tetris Rosy Retrospection.gb");
    let rom = std::fs::read(&from).ok()?;
    std::fs::write(root.join("Games/GB/Tetris Rosy Retrospection.gb"), rom)
        .expect("copy the card's cart");
    Some(())
}

fn run_for(f: &mut Frontend, input: &mut Script, secs: f32) {
    let until = Instant::now() + Duration::from_secs_f32(secs);
    while Instant::now() < until {
        f.advance(input);
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn l_widens_the_picture_like_a_gba_and_r_gives_back_the_centred_picture() {
    let _core = core_lock();
    let core = Core::default_for(Platform::Gb);
    let file = format!(
        "{}_libretro.{}",
        core.as_str(),
        std::env::consts::DLL_EXTENSION
    );
    let dylib = repo_root().join("vendor").join(&file);
    if !dylib.exists() {
        eprintln!("no {} dylib, skipping", core.as_str());
        return;
    }
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };

    let d = tmp_root_with_gb_carts(&["Zzz"]);
    let Some(()) = put_card_cart(d.path()) else {
        eprintln!("no Game Boy cart on this machine's card, skipping");
        return;
    };
    std::fs::copy(&dylib, d.path().join("System").join(&file)).expect("plant the core");
    clocked(d.path());

    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    let mut input = Script(VecDeque::new());
    input.0.push_back(vec![RawEvent::Down(Btn::A)]);
    f.advance(&mut input);
    input.0.push_back(vec![RawEvent::Up(Btn::A)]);
    f.advance(&mut input);
    run_for(&mut f, &mut input, 5.0);

    f.compose(&mut c);
    let actual = c.read_frame();
    write_png("picture-actual", OUT_W, OUT_H, &actual);

    let mid_y = OUT_H as usize / 2 - 40..OUT_H as usize / 2 + 40;
    assert!(
        lit(&actual, BAR_W..OUT_W as usize - BAR_W, mid_y.clone()) > 1000,
        "nothing is playing: the panel is dark where the picture should be"
    );
    assert_eq!(
        lit(&actual, 0..BAR_W, 0..OUT_H as usize),
        0,
        "actual size has something in the left margin"
    );
    assert_eq!(
        lit(
            &actual,
            OUT_W as usize - BAR_W..OUT_W as usize,
            0..OUT_H as usize
        ),
        0,
        "actual size has something in the right margin"
    );
    assert_eq!(
        lit(&actual, 0..OUT_W as usize, 0..BAR_H),
        0,
        "actual size has something above the picture"
    );

    input.0.push_back(vec![RawEvent::Down(Btn::L1)]);
    f.advance(&mut input);
    input.0.push_back(vec![RawEvent::Up(Btn::L1)]);
    run_for(&mut f, &mut input, 1.0);
    f.compose(&mut c);
    let stretched = c.read_frame();
    write_png("picture-stretch", OUT_W, OUT_H, &stretched);

    let left = lit(&stretched, 0..BAR_W, 0..OUT_H as usize);
    let top = lit(&stretched, 0..OUT_W as usize, 0..BAR_H);
    assert!(
        left > BAR_W * OUT_H as usize / 4,
        "the left margin is still mostly empty in fullscreen: {left} lit"
    );
    assert_eq!(
        top, 0,
        "the stretch grew the picture upward; a GBA keeps the 144 lines"
    );

    let (w, h, both) = side_by_side(&actual, &stretched, 8);
    write_png("picture-side-by-side", w, h, &both);
    let (w, h, close) = zoom(&actual, &stretched, 300, 200, 90, 60, 6);
    write_png("picture-zoom", w, h, &close);

    input.0.push_back(vec![RawEvent::Down(Btn::R1)]);
    f.advance(&mut input);
    input.0.push_back(vec![RawEvent::Up(Btn::R1)]);
    run_for(&mut f, &mut input, 1.0);
    f.compose(&mut c);
    let back = c.read_frame();
    write_png("picture-back", OUT_W, OUT_H, &back);
    assert_eq!(
        lit(&back, 0..BAR_W, 0..OUT_H as usize),
        0,
        "R did not give the margins back"
    );
    assert!(
        lit(&back, BAR_W..OUT_W as usize - BAR_W, mid_y) > 1000,
        "R left the panel dark"
    );
}

#[test]
fn retro_tiles_renderer_ignores_stretch_and_restores_other_effects() {
    let _core = core_lock();
    let surface = match HeadlessSurface::new() {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("headless GL unavailable, skipping: {error:?}");
            return;
        }
    };
    let mut c = Compositor::new(&surface).expect("compositor initialization failed");
    use slot::video_mode::{source_rect, VideoMode};
    use slot_gfx::{ScreenEffect, SRC_H, SRC_W};
    for platform in [Platform::Gba, Platform::Gb, Platform::Gbc] {
        let (w, h) = platform.picture();
        let (x0, y0) = ((SRC_W - w) / 2, (SRC_H - h) / 2);
        let picture = [
            x0 as f32 / SRC_W as f32,
            y0 as f32 / SRC_H as f32,
            (x0 + w) as f32 / SRC_W as f32,
            (y0 + h) as f32 / SRC_H as f32,
        ];
        let mut source = Vec::new();
        for y in 0..SRC_H {
            for x in 0..SRC_W {
                let grey = if (x0..x0 + w).contains(&x) && (y0..y0 + h).contains(&y) {
                    40 + ((x + y) % 100) as u8
                } else {
                    0
                };
                source.extend_from_slice(&[grey, grey, grey, 255]);
            }
        }
        for window in [(720, 480), (640, 480)] {
            let mut integer_picture = None;
            for mode in [VideoMode::Actual, VideoMode::Stretch] {
                c.fit(window);
                c.set_picture(picture);
                c.set_game_source_rect(source_rect(platform, mode));
                for previous in [ScreenEffect::None, ScreenEffect::Lcd3x, ScreenEffect::Grid] {
                    c.set_screen_effect(previous);
                    c.begin_frame();
                    c.upload_game(&source);
                    c.draw_game();
                    let before = c.read_window(window);
                    c.set_screen_effect(ScreenEffect::RetroTiles);
                    c.begin_frame();
                    c.draw_game();
                    let tiles = c.read_window(window);
                    assert_ne!(
                        before, tiles,
                        "RetroTiles did not change {platform:?} {mode:?}"
                    );
                    if let Some(ref expected) = integer_picture {
                        assert_eq!(
                            &tiles, expected,
                            "stretch changed RetroTiles on {platform:?}"
                        );
                    } else {
                        integer_picture = Some(tiles);
                    }
                    c.set_screen_effect(previous);
                    c.begin_frame();
                    c.draw_game();
                    assert_eq!(
                        before,
                        c.read_window(window),
                        "did not restore {previous:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn frontend_shader_shortcuts_map_retro_tiles_and_keep_the_changed_picture_mode() {
    use slot::app::Phase;
    use slot::video_mode::{source_rect, video_mode_for, VideoMode};
    use slot_gfx::{ScreenEffect, SRC_H, SRC_W};
    use slot_store::Shader;

    let _core = core_lock();
    let surface = match HeadlessSurface::new() {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("headless GL unavailable, skipping: {error:?}");
            return;
        }
    };
    let mut c = Compositor::new(&surface).expect("compositor initialization failed");
    let d = tmp_root_with_gb_carts(&["Tetris"]);
    clocked(d.path());
    // Force the existing mock fallback, independent of installed emulator cores.
    struct CoreOverride(Option<std::ffi::OsString>);
    impl Drop for CoreOverride {
        fn drop(&mut self) {
            match self.0.take() {
                Some(value) => std::env::set_var("SLOT_CORE", value),
                None => std::env::remove_var("SLOT_CORE"),
            }
        }
    }
    let _override = CoreOverride(std::env::var_os("SLOT_CORE"));
    std::env::set_var("SLOT_CORE", d.path().join("missing-core"));
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    let mut input = Script(VecDeque::new());
    let mut now = 0;
    let mut feed = |f: &mut Frontend, events: Vec<RawEvent>| {
        now += 16;
        input.0.push_back(events);
        f.advance_at(&mut input, now, 1.0 / 60.0);
    };
    feed(&mut f, vec![RawEvent::Down(Btn::A)]);
    feed(&mut f, vec![RawEvent::Up(Btn::A)]);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(f.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "mock game never started");
        f.compose(&mut c);
        feed(&mut f, vec![]);
        std::thread::sleep(Duration::from_millis(2));
    }
    for _ in 0..90 {
        feed(&mut f, vec![]);
    }
    for (button, mode) in [(Btn::L1, VideoMode::Stretch), (Btn::R1, VideoMode::Actual)] {
        // Default binding: Select+A selects the next shader, Select+B the previous.
        for (shortcut, shader, effect) in [
            (Btn::A, Shader::RetroTiles, ScreenEffect::RetroTiles),
            (Btn::B, Shader::Simpletex, ScreenEffect::Simpletex),
        ] {
            feed(&mut f, vec![RawEvent::Down(Btn::Select)]);
            feed(&mut f, vec![RawEvent::Down(shortcut)]);
            feed(
                &mut f,
                vec![RawEvent::Up(shortcut), RawEvent::Up(Btn::Select)],
            );
            assert_eq!(f.app().screen_shader(), shader);
            if shader == Shader::RetroTiles {
                feed(&mut f, vec![RawEvent::Down(button)]);
                feed(&mut f, vec![RawEvent::Up(button)]);
            }
            assert_eq!(video_mode_for(d.path(), "Tetris"), mode);
            assert_eq!(f.app().source_rect(), source_rect(Platform::Gb, mode));
            f.compose(&mut c);
            // Probe the effect installed by compose with a deterministic frame,
            // avoiding differences caused by the emulation worker or HUD animation.
            c.begin_frame();
            c.upload_game(&vec![96; (SRC_W * SRC_H * 4) as usize]);
            c.draw_game();
            let mapped = c.read_frame();
            c.set_screen_effect(effect);
            c.begin_frame();
            c.draw_game();
            assert_eq!(
                mapped,
                c.read_frame(),
                "frontend mapped {shader:?} incorrectly"
            );
            c.set_screen_effect(ScreenEffect::None);
            c.begin_frame();
            c.draw_game();
            assert_ne!(
                mapped,
                c.read_frame(),
                "probe must distinguish the selected effect from Off"
            );
        }
    }
}
