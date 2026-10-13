#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;
use std::time::Duration;

use common::tmp_root_with_carts;
use slot::frontend::Frontend;
use slot_gfx::{Compositor, HeadlessSurface, OUT_H, OUT_W};
use slot_input::{InputSource, Millis, RawEvent};
use slot_power::SimPlatform;
use slot_store::{read_slot_state, write_favorites, write_slot_state, Platform};

struct Still(VecDeque<Vec<RawEvent>>);

impl InputSource for Still {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

#[test]
fn the_favorites_shelf_opens_first_with_stars_on_its_carts() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Fusion", "Minish Cap", "Zelda"]);
    write_favorites(
        d.path(),
        &[
            (Platform::Gba, "Fusion".into()),
            (Platform::Gba, "Zelda".into()),
        ],
    )
    .expect("write favorites");
    let mut s = read_slot_state(d.path());
    s.clock_set = true;
    s.start_favorites = true;
    write_slot_state(d.path(), &s).expect("write slot.state");
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    let mut input = Still(VecDeque::new());
    f.advance(&mut input);
    f.compose(&mut c);
    std::thread::sleep(Duration::from_millis(300));
    f.advance(&mut input);
    f.compose(&mut c);
    let px = c.read_frame();
    if let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") {
        let file = std::fs::File::create(format!("{dir}/favorites.png")).expect("png");
        let mut e = png::Encoder::new(std::io::BufWriter::new(file), OUT_W, OUT_H);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&px).unwrap();
    }
    assert!(
        px.chunks(4).any(|p| p[0] > 230 && p[1] > 180 && p[2] < 120),
        "no gold star drawn"
    );
}
