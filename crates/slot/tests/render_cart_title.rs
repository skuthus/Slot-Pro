#![cfg(target_os = "macos")]

mod common;

use std::collections::VecDeque;
use std::path::Path;

use common::{tmp_root_with_carts, tmp_root_with_gb_carts};
use slot::frontend::Frontend;
use slot_gfx::{Compositor, HeadlessSurface, OUT_H, OUT_W};
use slot_gfx::{Draw, TexId};
use slot_input::{InputSource, Millis, RawEvent};
use slot_power::SimPlatform;
use slot_store::{read_slot_state, write_slot_state, CartTitle, Platform};
use slot_ui::{cart_box, rest_y, set_shelf_slack, MOUTH_H};

static SHELF: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Still(VecDeque<Vec<RawEvent>>);

impl InputSource for Still {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn frame(root: &Path, c: &mut Compositor, place: CartTitle, name: &str) -> Vec<u8> {
    let mut s = read_slot_state(root);
    s.clock_set = true;
    s.cart_title = place;
    write_slot_state(root, &s).expect("write slot.state");
    let mut f = Frontend::boot(Box::new(SimPlatform::at(root.to_path_buf())));
    f.upload_faces(c);
    let mut input = Still(VecDeque::new());
    f.advance(&mut input);
    f.compose(c);
    f.advance(&mut input);
    std::thread::sleep(std::time::Duration::from_millis(300));
    f.advance(&mut input);
    f.compose(c);
    let px = c.read_frame();
    if let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") {
        let path = format!("{dir}/title-{name}.png");
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

fn title_y(root: &Path, place: CartTitle) -> (f32, f32) {
    let mut s = read_slot_state(root);
    s.clock_set = true;
    s.cart_title = place;
    write_slot_state(root, &s).expect("write slot.state");
    let (mut a, _clock) = common::app_booting_at(root, common::CLOCK_IS_SET);
    let face = TexId::from_raw(9_001);
    a.set_cart_title_face(Some((face, 600, 30)));
    let mut out = Vec::new();
    a.draw(&mut out);
    out.iter()
        .find_map(|d| match *d {
            Draw::Tex { tex, y, h, .. } if tex == face => Some((y, y + h)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no title drawn for {place:?}"))
}

#[test]
fn the_title_sits_clear_of_the_cart_and_the_slot_on_every_shelf_and_panel() {
    let _shelf = SHELF.lock().unwrap_or_else(|e| e.into_inner());
    let gba = tmp_root_with_carts(&["Pokemon Emerald (USA)", "Metroid Fusion"]);
    let gb = tmp_root_with_gb_carts(&["Links Awakening DX", "Tetris"]);
    for slack in [0.0, 60.0] {
        set_shelf_slack(slack);
        for (root, platform) in [(gba.path(), Platform::Gba), (gb.path(), Platform::Gbc)] {
            for place in [CartTitle::Below, CartTitle::Above] {
                slot_ui::set_title_above(place == CartTitle::Above);
                let (top, bottom) = title_y(root, place);
                let (_, ch) = cart_box(platform);
                let cart_top = rest_y(ch as f32);
                let cart_foot = cart_top + ch as f32;
                let name = format!("{platform:?} {place:?} with {slack} slack");
                assert!(top >= 0.0, "{name}: the title is off the top at {top}");
                assert!(
                    bottom <= OUT_H as f32 - MOUTH_H,
                    "{name}: the title runs into the slot at {bottom}"
                );
                match place {
                    CartTitle::Below => assert!(
                        top >= cart_foot,
                        "{name}: the title at {top} overlaps the cart ending at {cart_foot}"
                    ),
                    _ => assert!(
                        bottom <= cart_top,
                        "{name}: the title ending at {bottom} overlaps the cart from {cart_top}"
                    ),
                }
            }
        }
    }
    set_shelf_slack(0.0);
    slot_ui::set_title_above(false);
}

#[test]
fn the_title_renders_on_each_shelf_for_a_look() {
    let _shelf = SHELF.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let gba = tmp_root_with_carts(&["Pokemon Emerald (USA)", "Metroid Fusion"]);
    let gb = tmp_root_with_gb_carts(&["Links Awakening DX", "Tetris"]);
    for (root, shelf) in [(gba.path(), "gba"), (gb.path(), "gb")] {
        let bare = frame(root, &mut c, CartTitle::Off, &format!("{shelf}-off"));
        for place in [CartTitle::Below, CartTitle::Above] {
            let titled = frame(root, &mut c, place, &format!("{shelf}-{}", place.name()));
            assert_ne!(bare, titled, "{shelf} {place:?}: nothing was drawn");
        }
    }
    slot_ui::set_title_above(false);
}
