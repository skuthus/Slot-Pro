#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;

use common::{clocked, tmp_root_with_carts, tmp_root_with_gb_carts};
use slot::frontend::Frontend;
use slot_gfx::{Compositor, HeadlessSurface, OUT_H, OUT_W};
use slot_input::{Btn, InputSource, Millis, RawEvent};
use slot_power::SimPlatform;
use slot_store::{Core, Platform, StateRing};

struct Script(VecDeque<Vec<RawEvent>>);

impl InputSource for Script {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn thumb(w: u32, h: u32) -> Vec<u8> {
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            rgb.extend([(x * 255 / w) as u8, (y * 255 / h) as u8, 0x90]);
        }
    }
    let mut out = Vec::new();
    {
        let mut e = png::Encoder::new(&mut out, w, h);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()
            .expect("png header")
            .write_image_data(&rgb)
            .expect("png data");
    }
    out
}

fn fill_slots(root: &Path, platform: Platform, stems: &[&str], size: (u32, u32)) {
    for stem in stems {
        let ring = StateRing::new(root, platform, Core::Mgba, stem);
        ring.write_slot(0, b"state", &thumb(size.0, size.1), "2026-10-12_09-00-00")
            .expect("write a slot");
    }
}

fn picker(root: &Path, c: &mut Compositor, name: &str) -> Vec<u8> {
    clocked(root);
    let mut f = Frontend::boot(Box::new(SimPlatform::at(root.to_path_buf())));
    f.upload_faces(c);
    let mut input = Script(VecDeque::new());
    f.advance(&mut input);
    f.compose(c);
    std::thread::sleep(Duration::from_millis(300));
    f.advance(&mut input);
    input.0.push_back(vec![RawEvent::Down(Btn::Select)]);
    f.advance(&mut input);
    std::thread::sleep(Duration::from_millis(60));
    input.0.push_back(vec![RawEvent::Up(Btn::Select)]);
    f.advance(&mut input);
    std::thread::sleep(Duration::from_millis(80));
    f.advance(&mut input);
    f.compose(c);
    f.advance(&mut input);
    f.compose(c);
    let px = c.read_frame();
    if let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") {
        let path = format!("{dir}/{name}.png");
        let file = std::fs::File::create(&path).expect("create png");
        let mut e = png::Encoder::new(std::io::BufWriter::new(file), OUT_W, OUT_H);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()
            .expect("png header")
            .write_image_data(&px)
            .expect("png data");
        println!("wrote {path}");
    }
    px
}

fn at(px: &[u8], x: u32, y: u32) -> [u8; 3] {
    let o = ((y * OUT_W + x) * 4) as usize;
    [px[o], px[o + 1], px[o + 2]]
}

#[test]
fn select_on_the_carousel_puts_the_slots_picture_where_the_cart_was() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let gba = tmp_root_with_carts(&["Emerald", "Fusion"]);
    fill_slots(
        gba.path(),
        Platform::Gba,
        &["Emerald", "Fusion"],
        (240, 160),
    );
    let px = picker(gba.path(), &mut c, "shelf-slots-gba");
    let [_, _, b] = at(&px, OUT_W / 2, 240);
    assert!(b > 0x70, "no screenshot where the GBA cart sits");

    let gb = tmp_root_with_gb_carts(&["Tetris", "Wario"]);
    fill_slots(gb.path(), Platform::Gb, &["Tetris", "Wario"], (160, 144));
    let px = picker(gb.path(), &mut c, "shelf-slots-gb");
    let [_, _, b] = at(&px, OUT_W / 2, 200);
    assert!(b > 0x70, "no screenshot where the GB cart sits");
}
