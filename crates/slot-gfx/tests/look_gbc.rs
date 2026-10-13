// Renders a test card through the GB screen shaders, side by side, for judging the look.
use slot_gfx::{Compositor, Draw, HeadlessSurface, ScreenEffect, OUT_H, OUT_W, SRC_H, SRC_W};

fn card() -> Vec<u8> {
    let (w, h) = (SRC_W as usize, SRC_H as usize);
    let mut px = vec![0u8; w * h * 4];
    let (x0, y0) = ((w - 160) / 2, (h - 144) / 2);
    let swatches: [[u8; 3]; 8] = [
        [248, 248, 248],
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [255, 255, 0],
        [255, 0, 255],
        [0, 255, 255],
        [0, 0, 0],
    ];
    for y in 0..144 {
        for x in 0..160 {
            let c = if y < 48 {
                swatches[x / 20]
            } else if y < 96 {
                let t = (x * 255 / 159) as u8;
                [t, (255 - t) / 2 + 40, 200 - t / 2]
            } else if (x / 8 + y / 8) % 2 == 0 {
                [16, 24, 32]
            } else {
                [232, 200, 120]
            };
            let o = ((y0 + y) * w + x0 + x) * 4;
            px[o..o + 4].copy_from_slice(&[c[2], c[1], c[0], 255]);
        }
    }
    px
}

#[test]
#[ignore]
fn gb_shaders_side_by_side() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let gb = [40.0 / 240.0, 8.0 / 160.0, 200.0 / 240.0, 152.0 / 160.0];
    let dir = std::env::var("SCRATCH_PNG_DIR").unwrap();
    for (name, effect) in [("off", ScreenEffect::None), ("gbc", ScreenEffect::GbcLcd)] {
        c.set_picture(gb);
        c.set_screen_effect(effect);
        c.begin_frame();
        c.upload_game(&card());
        c.draw_list(&[Draw::Game]);
        let px = c.read_frame();
        let f = std::fs::File::create(format!("{dir}/look-{name}.png")).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), OUT_W, OUT_H);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&px).unwrap();
    }
}
