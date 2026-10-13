use slot_gfx::{Draw, TexId, OUT_H, OUT_W};
use slot_store::Cart;
use slot_store::CartTitle;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::cart::{cart_box, gb_shell_of, label_colour, label_text, CART_H, CART_W};
use crate::hud::{Millis, HUD_INK};
use crate::icon::{haloed, HALO_PX};
use crate::silhouette::GbShell;
use crate::slot_chrome::{draw_empty_slot, MOUTH_H};
use crate::text;
use crate::CartFace;

const SIDE_SCALE: f32 = 0.78;
const SIDE_ALPHA: f32 = 0.55;

static SLACK: AtomicU32 = AtomicU32::new(0);
static TITLE_ABOVE: AtomicBool = AtomicBool::new(false);

const TITLE_W: u32 = 600;
const TITLE_H: u32 = 28;
const TITLE_PX: f32 = 20.0;
const TITLE_MIN_PX: f32 = 14.0;
const TITLE_GAP: f32 = 6.0;
const TITLE_ROOM: f32 = TITLE_H as f32 + 2.0 * HALO_PX as f32 + 2.0 * TITLE_GAP;

pub fn set_title_above(on: bool) {
    TITLE_ABOVE.store(on, Ordering::Relaxed);
}

pub fn cart_title_face(text: &str) -> CartFace {
    let Some(font) = text::label_font() else {
        return CartFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, text, TITLE_W as f32, 1, TITLE_PX, TITLE_MIN_PX);
    haloed(
        &text::coverage(TITLE_W, TITLE_H, &layout),
        TITLE_W,
        TITLE_H,
        HUD_INK,
    )
}

pub fn set_shelf_slack(px: f32) {
    SLACK.store(px.max(0.0).to_bits(), Ordering::Relaxed);
}

fn raise() -> f32 {
    f32::from_bits(SLACK.load(Ordering::Relaxed)) / 2.0
}

pub fn panel_middle_y() -> f32 {
    OUT_H as f32 / 2.0 + raise()
}

pub fn rest_y(h: f32) -> f32 {
    let y = if h > CART_H as f32 {
        (OUT_H as f32 - MOUTH_H - h) / 2.0
    } else {
        (OUT_H + CART_H) as f32 / 2.0 - h
    };
    let y = y - raise();
    if h > CART_H as f32 && TITLE_ABOVE.load(Ordering::Relaxed) {
        return y.max(TITLE_ROOM);
    }
    y
}

pub fn foot_y(h: f32) -> f32 {
    rest_y(h) + h
}
const OMEGA: f32 = 16.0;
const PART: f32 = 130.0;

const SLOTS: i32 = 3;

const REPEAT_DELAY_MS: Millis = 400;
const REPEAT_MS: [Millis; 4] = [110, 85, 65, 50];

pub struct Shelf {
    pub carts: Vec<Cart>,
    pub index: usize,
    pub scroll: f32,
    faces: Vec<Option<TexId>>,
    shadow: Option<TexId>,
    gb_shadow: Option<TexId>,
    gbc_shadow: Option<TexId>,
    shells: Vec<Option<GbShell>>,
    starred: Vec<bool>,
    star: Option<TexId>,
    ride: f32,
    vel: f32,
    held: Option<(i32, Millis, usize)>,
}

impl Shelf {
    pub fn new(carts: Vec<Cart>) -> Self {
        Shelf {
            shells: carts.iter().map(gb_shell_of).collect(),
            faces: vec![None; carts.len()],
            starred: vec![false; carts.len()],
            star: None,
            carts,
            index: 0,
            scroll: 0.0,
            shadow: None,
            gb_shadow: None,
            gbc_shadow: None,
            ride: 0.0,
            vel: 0.0,
            held: None,
        }
    }

    pub fn set_carts(&mut self, carts: Vec<Cart>) -> Vec<TexId> {
        let shed = self.faces.drain(..).flatten().collect();
        self.shells = carts.iter().map(gb_shell_of).collect();
        self.faces = vec![None; carts.len()];
        self.starred = vec![false; carts.len()];
        self.carts = carts;
        self.index = self.index.min(self.carts.len().saturating_sub(1));
        self.scroll = self.scroll_target();
        self.vel = 0.0;
        self.held = None;
        shed
    }

    pub fn set_starred(&mut self, starred: Vec<bool>) {
        self.starred = starred;
    }

    pub fn set_star(&mut self, face: TexId) {
        self.star = Some(face);
    }

    pub fn select(&mut self, i: usize) {
        self.index = i;
        self.scroll = i as f32;
        self.ride = i as f32;
        self.vel = 0.0;
    }

    pub fn set_shadow(&mut self, face: TexId) {
        self.shadow = Some(face);
    }

    pub fn set_gb_shadow(&mut self, shell: GbShell, face: TexId) {
        match shell {
            GbShell::Notched => self.gb_shadow = Some(face),
            GbShell::Rounded => self.gbc_shadow = Some(face),
        }
    }

    pub fn set_faces(&mut self, faces: Vec<TexId>) {
        let mut faces = faces.into_iter();
        for face in &mut self.faces {
            *face = faces.next();
        }
    }

    pub fn face(&self, i: usize) -> Option<TexId> {
        self.faces.get(i).copied().flatten()
    }

    pub fn set_face(&mut self, i: usize, tex: TexId) -> Option<TexId> {
        self.faces.get_mut(i).and_then(|f| f.replace(tex))
    }

    pub fn take_face(&mut self, i: usize) -> Option<TexId> {
        self.faces.get_mut(i).and_then(Option::take)
    }

    pub fn on_screen(&self) -> Vec<usize> {
        self.around(SLOTS)
    }

    pub fn around(&self, radius: i32) -> Vec<usize> {
        let mut out = Vec::new();
        for step in 0..=radius {
            for off in [step, -step] {
                if let Some(i) = self.cart_at_offset(off) {
                    if !out.contains(&i) {
                        out.push(i);
                    }
                }
            }
        }
        out
    }

    pub fn find(&self, stem: &str) -> Option<(&Cart, Option<TexId>)> {
        let i = self.carts.iter().position(|c| c.stem == stem)?;
        Some((&self.carts[i], self.face(i)))
    }

    pub fn left(&mut self) {
        self.step(-1);
    }

    pub fn right(&mut self) {
        self.step(1);
    }

    pub fn jump_next_letter(&mut self) {
        self.jump(1);
    }

    pub fn jump_prev_letter(&mut self) {
        self.jump(-1);
    }

    fn start_of_letter(&self, from: usize) -> usize {
        let n = self.carts.len();
        let letter = slot_store::initial(&self.carts[from].stem);
        let mut at = from;
        for _ in 0..n {
            let before = (at as i32 - 1).rem_euclid(n as i32) as usize;
            if slot_store::initial(&self.carts[before].stem) != letter {
                break;
            }
            at = before;
        }
        at
    }

    fn jump(&mut self, dir: i32) {
        let n = self.carts.len();
        if n < 2 {
            return;
        }
        let wrap = |i: i32| i.rem_euclid(n as i32) as usize;
        let here = slot_store::initial(&self.carts[self.index].stem);
        if self
            .carts
            .iter()
            .all(|c| slot_store::initial(&c.stem) == here)
        {
            return;
        }
        let target = match dir > 0 {
            true => {
                let mut at = self.index;
                for _ in 0..n {
                    at = wrap(at as i32 + 1);
                    if slot_store::initial(&self.carts[at].stem) != here {
                        break;
                    }
                }
                at
            }
            false => {
                let start = self.start_of_letter(self.index);
                match start == self.index {
                    true => self.start_of_letter(wrap(start as i32 - 1)),
                    false => start,
                }
            }
        };
        let ahead = (target as i32 - self.index as i32).rem_euclid(n as i32);
        let delta = match dir > 0 {
            true => ahead,
            false => ahead - n as i32,
        };
        self.index = target;
        self.ride += delta as f32;
    }

    pub fn hold_left(&mut self, now: Millis) {
        self.hold(-1, now);
    }

    pub fn hold_right(&mut self, now: Millis) {
        self.hold(1, now);
    }

    fn hold(&mut self, by: i32, now: Millis) {
        self.step(by);
        self.held = Some((by, now + REPEAT_DELAY_MS, 0));
    }

    pub fn release_left(&mut self) {
        self.release(-1);
    }

    pub fn release_right(&mut self) {
        self.release(1);
    }

    fn release(&mut self, by: i32) {
        if matches!(self.held, Some((held, _, _)) if held == by) {
            self.held = None;
        }
    }

    pub fn release_hold(&mut self) {
        self.held = None;
    }

    pub fn tick(&mut self, now: Millis) {
        let Some((by, due, fired)) = self.held else {
            return;
        };
        if now < due {
            return;
        }
        self.step(by);
        let rate = REPEAT_MS[fired.min(REPEAT_MS.len() - 1)];
        self.held = Some((by, now + rate, fired + 1));
    }

    fn step(&mut self, by: i32) {
        let n = self.carts.len();
        if n < 2 {
            return;
        }
        self.index = (self.index as i32 + by).rem_euclid(n as i32) as usize;
        self.ride += by as f32;
    }

    pub fn scroll_target(&self) -> f32 {
        let n = self.carts.len();
        if n == 0 {
            return 0.0;
        }
        let from = self.ride;
        let n = n as f32;
        from + (self.index as f32 - from + n / 2.0).rem_euclid(n) - n / 2.0
    }

    pub fn cart_at_offset(&self, off: i32) -> Option<usize> {
        let n = self.carts.len() as i32;
        if n == 0 {
            return None;
        }
        if n == 1 {
            return (off == 0).then_some(self.index);
        }
        Some((self.index as i32 + off).rem_euclid(n) as usize)
    }

    pub fn selected_at(&self) -> (f32, f32) {
        let w = self
            .carts
            .get(self.index)
            .map_or(CART_W, |c| cart_box(c.platform).0) as f32;
        let offset = self.scroll_target() - self.scroll;
        let scale = shrink(offset);
        (OUT_W as f32 / 2.0 + offset * w - w * scale / 2.0, scale)
    }

    pub fn update(&mut self, dt: f32) {
        let accel = -2.0 * OMEGA * self.vel - OMEGA * OMEGA * (self.scroll - self.scroll_target());
        self.vel += accel * dt;
        self.scroll += self.vel * dt;
    }

    pub fn draw_title(
        &self,
        face: (TexId, u32, u32),
        place: CartTitle,
        shake: f32,
        out: &mut Vec<Draw>,
    ) {
        let Some(cart) = self.carts.get(self.index) else {
            return;
        };
        let alpha = 1.0 - (self.scroll_target() - self.scroll).abs().min(1.0);
        if alpha <= 0.0 {
            return;
        }
        let (tex, w, h) = face;
        let (_, ch) = cart_box(cart.platform);
        let ch = ch as f32;
        let y = match place {
            CartTitle::Off => return,
            CartTitle::Below => foot_y(ch) + TITLE_GAP,
            CartTitle::Above => rest_y(ch) - TITLE_GAP - h as f32,
        };
        out.push(Draw::Tex {
            x: ((OUT_W as f32 - w as f32) / 2.0 + shake).round(),
            y: y.max(0.0).round(),
            w: w as f32,
            h: h as f32,
            tex,
            alpha,
        });
    }

    pub fn draw(&self, shake: f32, out: &mut Vec<Draw>) {
        self.draw_row(None, shake, 0.0, 1.0, out);
        draw_empty_slot(out);
    }

    pub fn draw_row(
        &self,
        hidden: Option<&str>,
        shake: f32,
        recede: f32,
        dim: f32,
        out: &mut Vec<Draw>,
    ) {
        let recede = recede.clamp(0.0, 1.0);
        let dim = dim.clamp(0.0, 1.0);
        let target = self.scroll_target();
        for slot in -SLOTS..=SLOTS {
            let Some(i) = self.cart_at_offset(slot) else {
                continue;
            };
            let cart = &self.carts[i];
            if hidden == Some(cart.stem.as_str()) {
                continue;
            }
            let offset = target + slot as f32 - self.scroll;
            let t = offset.abs().min(1.0);
            let scale = shrink(offset);
            let alpha = (1.0 + (SIDE_ALPHA - 1.0) * t) * (1.0 - recede);
            let (cw, ch) = cart_box(cart.platform);
            let (w, h) = (cw as f32 * scale, ch as f32 * scale);
            let away = offset.signum() * (1.0 + offset.abs());
            let x = OUT_W as f32 / 2.0 + offset * cw as f32 - w / 2.0 + away * PART * recede;
            if x + w <= 0.0 || x >= OUT_W as f32 || alpha <= 0.0 {
                continue;
            }
            let x = x + shake;
            let y = foot_y(ch as f32) - h;
            if alpha < 1.0 {
                let backing = match self.shells.get(i).copied().flatten() {
                    None => self.shadow,
                    Some(GbShell::Notched) => self.gb_shadow,
                    Some(GbShell::Rounded) => self.gbc_shadow,
                };
                if let Some(tex) = backing {
                    out.push(Draw::Tex {
                        x,
                        y,
                        w,
                        h,
                        tex,
                        alpha: recede_alpha(alpha),
                    });
                }
            }
            out.push(match self.face(i) {
                Some(tex) => Draw::Tex {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha: alpha * dim,
                },
                None => {
                    let c = label_colour(&label_text(cart));
                    Draw::Rect {
                        x,
                        y,
                        w,
                        h,
                        colour: [
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0,
                            alpha * dim,
                        ],
                    }
                }
            });
            if let (Some(tex), Some(true)) = (self.star, self.starred.get(i).copied()) {
                let size = STAR_PX * scale;
                out.push(Draw::Tex {
                    x: x + w - size * 0.8,
                    y: y - size * 0.3,
                    w: size,
                    h: size,
                    tex,
                    alpha: alpha * dim,
                });
            }
        }
    }
}

pub const STAR_PX: f32 = 44.0;

pub fn star_face() -> crate::plate::UndoFace {
    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
<path d="M50 6 L62 37 L95 38 L69 58 L78 91 L50 72 L22 91 L31 58 L5 38 L38 37 Z"
 fill="#ffd23f" stroke="#1a1917" stroke-width="7" stroke-linejoin="round"/>
</svg>"##;
    let side = STAR_PX as u32 * 2;
    let rgba = crate::art::render_svg(SVG, side, side).unwrap_or_default();
    crate::plate::UndoFace {
        rgba,
        w: side,
        h: side,
    }
}

fn shrink(offset: f32) -> f32 {
    1.0 + (SIDE_SCALE - 1.0) * offset.abs().min(1.0)
}

fn recede_alpha(face_alpha: f32) -> f32 {
    (face_alpha / SIDE_ALPHA).clamp(0.0, 1.0)
}
