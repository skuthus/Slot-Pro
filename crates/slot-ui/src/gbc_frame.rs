use slot_gfx::{OUT_H, OUT_W};
use slot_store::GbFrame;

use crate::art::{decode_bytes, render_svg};
use crate::plate::UndoFace;
use crate::text;

pub const GB_FRAME_SCREEN: [f32; 4] = [120.0, 24.0, 480.0, 432.0];

const SIDE_MIDDLE: f32 = 52.0;
const LED_Y: f32 = 64.0;
const POWER_Y: f32 = 84.0;
const POWER_PX: f32 = 11.0;
const LOGO_PNG: &[u8] = include_bytes!("../assets/gbc_color.png");
const LOGO_FOOT: f32 = 16.0;
const LOGO_OPAQUE: u32 = 150;
const WORD_W: u32 = 100;
const WORD_H: u32 = 30;

const LIGHT_INK: [u8; 3] = [0xf2, 0xf2, 0xf6];
const DARK_INK: [u8; 3] = [0x2a, 0x2a, 0x31];

struct Shell {
    fill: [u8; 3],
    light: bool,
}

fn shell(frame: GbFrame) -> Option<Shell> {
    let (fill, light) = match frame {
        GbFrame::Off => return None,
        GbFrame::Berry => ([0xb5, 0x12, 0x4a], false),
        GbFrame::Grape => ([0x5a, 0x2d, 0x8c], false),
        GbFrame::Kiwi => ([0x8c, 0xc6, 0x3f], true),
        GbFrame::Dandelion => ([0xf2, 0xcc, 0x1a], true),
        GbFrame::Teal => ([0x00, 0xa3, 0xb0], false),
        GbFrame::AtomicPurple => ([0x6c, 0x5d, 0x93], false),
    };
    Some(Shell { fill, light })
}

pub fn gbc_frame_face(frame: GbFrame) -> Option<UndoFace> {
    let shell = shell(frame)?;
    let [r, g, b] = shell.fill;
    let [sx, sy, sw, sh] = GB_FRAME_SCREEN;
    let mut holes = String::new();
    for row in 0..6 {
        for col in 0..3 {
            let x = OUT_W as f32 - SIDE_MIDDLE + (col as f32 - 1.0) * 16.0 + (row % 2) as f32 * 8.0
                - 4.0;
            let y = 300.0 + row as f32 * 14.0;
            holes.push_str(&format!(
                r##"<circle cx="{x}" cy="{y}" r="4" fill="#000000" opacity="0.32"/>"##
            ));
        }
    }
    let led_x = SIDE_MIDDLE;
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{OUT_W}" height="{OUT_H}" viewBox="0 0 {OUT_W} {OUT_H}">
<defs><linearGradient id="sheen" x1="0" y1="0" x2="1" y2="1">
<stop offset="0" stop-color="#ffffff" stop-opacity="0.18"/>
<stop offset="1" stop-color="#000000" stop-opacity="0.22"/>
</linearGradient></defs>
<rect width="{OUT_W}" height="{OUT_H}" fill="rgb({r},{g},{b})"/>
<rect width="{OUT_W}" height="{OUT_H}" fill="url(#sheen)"/>
<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" rx="14" fill="#2a2a31"/>
<rect x="{ix}" y="{iy}" width="{iw}" height="{ih}" rx="12" fill="none" stroke="#45454f" stroke-width="2"/>
<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="2" fill="#0b0b0d"/>
<circle cx="{led_x}" cy="{LED_Y}" r="12" fill="#e8283c" opacity="0.22"/>
<circle cx="{led_x}" cy="{LED_Y}" r="6" fill="#e8283c"/>
{holes}
</svg>"##,
        lx = sx - 16.0,
        ly = sy - 16.0,
        lw = sw + 32.0,
        lh = sh + 32.0,
        ix = sx - 13.0,
        iy = sy - 13.0,
        iw = sw + 26.0,
        ih = sh + 26.0,
        bx = sx - 4.0,
        by = sy - 4.0,
        bw = sw + 8.0,
        bh = sh + 8.0,
    );
    let mut rgba = render_svg(&svg, OUT_W, OUT_H)?;
    if let Some(font) = text::label_font() {
        let ink = if shell.light { DARK_INK } else { LIGHT_INK };
        word(&mut rgba, font, "Power", POWER_Y, POWER_PX, |_| ink);
    }
    logo(&mut rgba);
    Some(UndoFace {
        rgba,
        w: OUT_W,
        h: OUT_H,
    })
}

fn logo(rgba: &mut [u8]) {
    let Some((art, w, h)) = decode_bytes(LOGO_PNG) else {
        return;
    };
    let left = (SIDE_MIDDLE - w as f32 / 2.0).round() as u32;
    let top = (OUT_H as f32 - LOGO_FOOT - h as f32).round() as u32;
    for y in 0..h {
        for x in 0..w {
            let px = &art[((y * w + x) * 4) as usize..][..4];
            let ink = 255 - px[..3].iter().copied().min().unwrap_or(255) as u32;
            let a = (ink * 255 / LOGO_OPAQUE).min(255) * px[3] as u32 / 255;
            if a == 0 {
                continue;
            }
            let at = (((top + y) * OUT_W + left + x) * 4) as usize;
            for c in 0..3 {
                let white = 255 * (255 - a);
                let pure = ((px[c] as u32 * 255).saturating_sub(white) / a).min(255);
                let under = rgba[at + c] as u32;
                rgba[at + c] = ((pure * a + under * (255 - a)) / 255) as u8;
            }
        }
    }
}

fn word(
    rgba: &mut [u8],
    font: &fontdue::Font,
    line: &str,
    top: f32,
    px: f32,
    ink: impl Fn(usize) -> [u8; 3],
) {
    let layout = text::fit(font, line, WORD_W as f32, 1, px, px * 0.6);
    let cov = text::coverage(WORD_W, WORD_H, &layout);
    let shown = line.to_uppercase();
    let set = text::line_width(font, &shown, layout.px, layout.tracking);
    let mut edges = Vec::new();
    let mut pen = (WORD_W as f32 - set) / 2.0;
    for ch in shown.chars() {
        edges.push(pen);
        pen += font.metrics(ch, layout.px).advance_width + layout.tracking;
    }
    let letter = |x: u32| {
        edges
            .iter()
            .rposition(|&e| x as f32 + 0.5 >= e)
            .unwrap_or(0)
    };
    let left = (SIDE_MIDDLE - WORD_W as f32 / 2.0).round() as i32;
    let top = (top - WORD_H as f32 / 2.0).round() as i32;
    for y in 0..WORD_H {
        for x in 0..WORD_W {
            let a = cov[(y * WORD_W + x) as usize] as u32;
            let (dx, dy) = (left + x as i32, top + y as i32);
            if a == 0 || dx < 0 || dy < 0 || dx >= OUT_W as i32 || dy >= OUT_H as i32 {
                continue;
            }
            let at = ((dy as u32 * OUT_W + dx as u32) * 4) as usize;
            let colour = ink(letter(x));
            for c in 0..3 {
                let under = rgba[at + c] as u32;
                rgba[at + c] = ((colour[c] as u32 * a + under * (255 - a)) / 255) as u8;
            }
        }
    }
}
