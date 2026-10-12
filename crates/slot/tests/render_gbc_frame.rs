#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use common::{clocked, core_lock, repo_root, tmp_root_with_gb_carts};
use slot::frontend::Frontend;
use slot_gfx::{Compositor, HeadlessSurface, OUT_H, OUT_W};
use slot_input::{InputSource, Millis, RawEvent};
use slot_power::SimPlatform;
use slot_store::{read_slot_state, write_slot_state, Core, GbFrame, Platform};

struct Script(VecDeque<Vec<RawEvent>>);

impl InputSource for Script {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn write_png(name: &str, rgba: &[u8]) {
    let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") else {
        return;
    };
    let path = format!("{dir}/{name}.png");
    let file = std::fs::File::create(&path).expect("create png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), OUT_W, OUT_H);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .expect("png header")
        .write_image_data(rgba)
        .expect("png data");
    println!("wrote {path}");
}

fn run_for(f: &mut Frontend, input: &mut Script, secs: f32) {
    let until = Instant::now() + Duration::from_secs_f32(secs);
    while Instant::now() < until {
        f.advance(input);
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn shot(f: &mut Frontend, c: &mut Compositor, name: &str) -> Vec<u8> {
    f.compose(c);
    let px = c.read_frame();
    write_png(name, &px);
    px
}

const LOGO: [u8; 48] = [
    0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d,
    0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99,
    0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
];

#[test]
fn a_gb_game_plays_inside_a_gbc_frame_in_each_shell() {
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
    for frame in [GbFrame::Grape, GbFrame::Teal, GbFrame::Dandelion] {
        let d = tmp_root_with_gb_carts(&["Framed"]);
        let mut rom = vec![0u8; 0x8000];
        rom[0x100..0x104].copy_from_slice(&[0x00, 0x18, 0xfe, 0x00]);
        rom[0x104..0x134].copy_from_slice(&LOGO);
        rom[0x14d] = rom[0x134..0x14d]
            .iter()
            .fold(0u8, |x, b| x.wrapping_sub(*b).wrapping_sub(1));
        std::fs::write(d.path().join("Games/GB/Framed.gb"), rom).expect("write a cart");
        std::fs::copy(&dylib, d.path().join("System").join(&file)).expect("plant the core");
        clocked(d.path());
        let mut state = read_slot_state(d.path());
        state.gb_frame = frame;
        write_slot_state(d.path(), &state).expect("write slot.state");

        let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
        f.upload_faces(&mut c);
        let mut input = Script(VecDeque::new());
        run_for(&mut f, &mut input, 3.0);
        f.compose(&mut c);
        let px = shot(&mut f, &mut c, &format!("gbc-frame-{}", frame.name()));
        let corner = &px[..4];
        assert!(
            corner[..3].iter().any(|&v| v > 40),
            "{frame:?}: no shell in the corner"
        );
    }
}
