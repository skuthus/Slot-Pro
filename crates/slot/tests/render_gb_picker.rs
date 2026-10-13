#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use common::{clocked, tmp_root_with_gb_carts};
use slot::frontend::Frontend;
use slot_gfx::{Compositor, HeadlessSurface, OUT_H, OUT_W};
use slot_input::{Btn, InputSource, Millis, RawEvent};
use slot_power::SimPlatform;

struct Script(VecDeque<Vec<RawEvent>>);

impl InputSource for Script {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn run_for(f: &mut Frontend, c: &mut Compositor, input: &mut Script, secs: f32) {
    let until = Instant::now() + Duration::from_secs_f32(secs);
    while Instant::now() < until {
        f.advance(input);
        f.compose(c);
        std::thread::sleep(Duration::from_millis(8));
    }
}

fn shot(c: &mut Compositor, name: &str) -> Vec<u8> {
    let px = c.read_frame();
    if let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") {
        let file = std::fs::File::create(format!("{dir}/{name}.png")).expect("png");
        let mut e = png::Encoder::new(std::io::BufWriter::new(file), OUT_W, OUT_H);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&px).unwrap();
    }
    px
}

#[test]
fn start_on_a_game_boy_cart_offers_mgba_and_gambatte() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_gb_carts(&["Tetris", "Wario"]);
    clocked(d.path());
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    let mut input = Script(VecDeque::new());
    run_for(&mut f, &mut c, &mut input, 0.5);
    input.0.push_back(vec![RawEvent::Down(Btn::Start)]);
    input.0.push_back(vec![RawEvent::Up(Btn::Start)]);
    run_for(&mut f, &mut c, &mut input, 1.5);
    shot(&mut c, "gb-picker");
    input.0.push_back(vec![RawEvent::Down(Btn::Right)]);
    input.0.push_back(vec![RawEvent::Up(Btn::Right)]);
    run_for(&mut f, &mut c, &mut input, 1.0);
    shot(&mut c, "gb-picker-gambatte");
}
