use crate::draw::{Draw, TexId, OUT_H, OUT_W};
use crate::plate::{arrows_hint_face, centred_hints, hint_face, UndoFace, HINT_H, LEGEND_GAP};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::{edge, opening};
use crate::text;
use slot_store::PadKey;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickRow {
    FastForward,
    FastForwardSound,
    Screen,
    Rumble,
    DateTime,
    About,
    GbaShader,
    GbShader,
    ColourCorrection,
    Game,
    EjectSave,
    Turbo,
    Rewind,
    GbPalettes,
    CartTitle,
    ClockFormat,
    GbFrame,
    Analog,
    Controls,
    RemapA,
    RemapB,
    RemapL,
    RemapR,
    RemapSelect,
    RemapStart,
    RemapX,
    RemapY,
    Resume,
    SaveState,
    LoadState,
    Shader,
    GameControls,
    EjectGame,
    GameLayout,
    GameRemapA,
    GameRemapB,
    GameRemapL,
    GameRemapR,
    GameRemapSelect,
    GameRemapStart,
    GameRemapX,
    GameRemapY,
}

impl QuickRow {
    pub const ALL: [QuickRow; 42] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Screen,
        QuickRow::Rumble,
        QuickRow::DateTime,
        QuickRow::About,
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::ColourCorrection,
        QuickRow::Game,
        QuickRow::EjectSave,
        QuickRow::Turbo,
        QuickRow::Rewind,
        QuickRow::GbPalettes,
        QuickRow::CartTitle,
        QuickRow::ClockFormat,
        QuickRow::GbFrame,
        QuickRow::Analog,
        QuickRow::Controls,
        QuickRow::RemapA,
        QuickRow::RemapB,
        QuickRow::RemapL,
        QuickRow::RemapR,
        QuickRow::RemapSelect,
        QuickRow::RemapStart,
        QuickRow::RemapX,
        QuickRow::RemapY,
        QuickRow::Resume,
        QuickRow::SaveState,
        QuickRow::LoadState,
        QuickRow::Shader,
        QuickRow::GameControls,
        QuickRow::EjectGame,
        QuickRow::GameLayout,
        QuickRow::GameRemapA,
        QuickRow::GameRemapB,
        QuickRow::GameRemapL,
        QuickRow::GameRemapR,
        QuickRow::GameRemapSelect,
        QuickRow::GameRemapStart,
        QuickRow::GameRemapX,
        QuickRow::GameRemapY,
    ];

    pub const MAIN: [QuickRow; 5] = [
        QuickRow::Screen,
        QuickRow::Game,
        QuickRow::Controls,
        QuickRow::DateTime,
        QuickRow::About,
    ];

    pub const SCREEN: [QuickRow; 7] = [
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::ColourCorrection,
        QuickRow::GbPalettes,
        QuickRow::GbFrame,
        QuickRow::CartTitle,
        QuickRow::ClockFormat,
    ];

    pub const GAME: [QuickRow; 7] = [
        QuickRow::Analog,
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Rewind,
        QuickRow::Turbo,
        QuickRow::Rumble,
        QuickRow::EjectSave,
    ];

    pub const CONTROLS: [QuickRow; 8] = [
        QuickRow::RemapA,
        QuickRow::RemapB,
        QuickRow::RemapL,
        QuickRow::RemapR,
        QuickRow::RemapSelect,
        QuickRow::RemapStart,
        QuickRow::RemapX,
        QuickRow::RemapY,
    ];

    pub const PAUSE: [QuickRow; 6] = [
        QuickRow::Resume,
        QuickRow::SaveState,
        QuickRow::LoadState,
        QuickRow::Shader,
        QuickRow::GameControls,
        QuickRow::EjectGame,
    ];

    pub const GAME_CONTROLS: [QuickRow; 9] = [
        QuickRow::GameLayout,
        QuickRow::GameRemapA,
        QuickRow::GameRemapB,
        QuickRow::GameRemapL,
        QuickRow::GameRemapR,
        QuickRow::GameRemapSelect,
        QuickRow::GameRemapStart,
        QuickRow::GameRemapX,
        QuickRow::GameRemapY,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn remaps(self) -> Option<PadKey> {
        Some(match self {
            QuickRow::RemapA | QuickRow::GameRemapA => PadKey::A,
            QuickRow::RemapB | QuickRow::GameRemapB => PadKey::B,
            QuickRow::RemapL | QuickRow::GameRemapL => PadKey::L,
            QuickRow::RemapR | QuickRow::GameRemapR => PadKey::R,
            QuickRow::RemapSelect | QuickRow::GameRemapSelect => PadKey::Select,
            QuickRow::RemapStart | QuickRow::GameRemapStart => PadKey::Start,
            QuickRow::RemapX | QuickRow::GameRemapX => PadKey::X,
            QuickRow::RemapY | QuickRow::GameRemapY => PadKey::Y,
            _ => return None,
        })
    }

    pub fn page(self) -> &'static [QuickRow] {
        match self {
            QuickRow::GbaShader
            | QuickRow::GbShader
            | QuickRow::ColourCorrection
            | QuickRow::GbPalettes
            | QuickRow::CartTitle
            | QuickRow::GbFrame
            | QuickRow::ClockFormat => &QuickRow::SCREEN,
            QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::Rewind
            | QuickRow::Turbo
            | QuickRow::Rumble
            | QuickRow::EjectSave
            | QuickRow::Analog => &QuickRow::GAME,
            QuickRow::RemapA
            | QuickRow::RemapB
            | QuickRow::RemapL
            | QuickRow::RemapR
            | QuickRow::RemapSelect
            | QuickRow::RemapStart
            | QuickRow::RemapX
            | QuickRow::RemapY => &QuickRow::CONTROLS,
            QuickRow::Resume
            | QuickRow::SaveState
            | QuickRow::LoadState
            | QuickRow::Shader
            | QuickRow::GameControls
            | QuickRow::EjectGame => &QuickRow::PAUSE,
            QuickRow::GameLayout
            | QuickRow::GameRemapA
            | QuickRow::GameRemapB
            | QuickRow::GameRemapL
            | QuickRow::GameRemapR
            | QuickRow::GameRemapSelect
            | QuickRow::GameRemapStart
            | QuickRow::GameRemapX
            | QuickRow::GameRemapY => &QuickRow::GAME_CONTROLS,
            _ => &QuickRow::MAIN,
        }
    }

    pub fn parent(self) -> Option<QuickRow> {
        match self.page()[0] {
            QuickRow::GbaShader => Some(QuickRow::Screen),
            QuickRow::Analog => Some(QuickRow::Game),
            QuickRow::RemapA => Some(QuickRow::Controls),
            QuickRow::GameLayout => Some(QuickRow::GameControls),
            _ => None,
        }
    }

    pub fn child(self) -> Option<QuickRow> {
        match self {
            QuickRow::Screen => Some(QuickRow::SCREEN[0]),
            QuickRow::Game => Some(QuickRow::GAME[0]),
            QuickRow::Controls => Some(QuickRow::CONTROLS[0]),
            QuickRow::GameControls => Some(QuickRow::GAME_CONTROLS[0]),
            _ => None,
        }
    }

    pub fn position(self) -> usize {
        self.page().iter().position(|r| *r == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            QuickRow::FastForward => "Fast Forward",
            QuickRow::FastForwardSound => "Fast Forward Sound",
            QuickRow::Screen => "Screen",
            QuickRow::Rumble => "Rumble",
            QuickRow::DateTime => "Date & Time",
            QuickRow::About => "About",
            QuickRow::GbaShader => "GBA Shader",
            QuickRow::GbShader => "GB / GBC Shader",
            QuickRow::ColourCorrection => "Color Correction",
            QuickRow::Game => "Gameplay",
            QuickRow::EjectSave => "Auto Save on Eject",
            QuickRow::Turbo => "Turbo Buttons",
            QuickRow::Rewind => "Rewind",
            QuickRow::GbPalettes => "GB Palettes",
            QuickRow::CartTitle => "Game Title",
            QuickRow::ClockFormat => "Clock",
            QuickRow::GbFrame => "GBC Frame",
            QuickRow::Analog => "Analog Mode",
            QuickRow::Controls | QuickRow::GameControls => "Controls",
            QuickRow::RemapA | QuickRow::GameRemapA => "A Button",
            QuickRow::RemapB | QuickRow::GameRemapB => "B Button",
            QuickRow::RemapL | QuickRow::GameRemapL => "L Button",
            QuickRow::RemapR | QuickRow::GameRemapR => "R Button",
            QuickRow::RemapSelect | QuickRow::GameRemapSelect => "Select",
            QuickRow::RemapStart | QuickRow::GameRemapStart => "Start",
            QuickRow::RemapX | QuickRow::GameRemapX => "X Button",
            QuickRow::RemapY | QuickRow::GameRemapY => "Y Button",
            QuickRow::Resume => "Resume",
            QuickRow::SaveState => "Save State",
            QuickRow::LoadState => "Load State",
            QuickRow::Shader => "Shader",
            QuickRow::EjectGame => "Eject Game",
            QuickRow::GameLayout => "Layout",
        }
    }

    pub fn opens(self) -> bool {
        matches!(
            self,
            QuickRow::DateTime
                | QuickRow::About
                | QuickRow::Screen
                | QuickRow::Game
                | QuickRow::Controls
                | QuickRow::GameControls
                | QuickRow::Resume
                | QuickRow::SaveState
                | QuickRow::LoadState
                | QuickRow::EjectGame
        )
    }

    pub fn up(self) -> QuickRow {
        let page = self.page();
        page[(self.position() + page.len() - 1) % page.len()]
    }

    pub fn down(self) -> QuickRow {
        let page = self.page();
        page[(self.position() + 1) % page.len()]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    GbaOnly,
    GbAndGbc,
    All,
    Below,
    Above,
    Hours12,
    Hours24,
    KeyA,
    KeyB,
    KeyL,
    KeyR,
    KeySelect,
    KeyStart,
    KeyX,
    KeyY,
    Berry,
    Grape,
    Kiwi,
    Dandelion,
    Teal,
    AtomicPurple,
    Global,
    Custom,
}

impl QuickValue {
    pub const ALL: [QuickValue; 33] = [
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
        QuickValue::Lcd3x,
        QuickValue::Grid,
        QuickValue::Dot,
        QuickValue::Simpletex,
        QuickValue::GbaOnly,
        QuickValue::GbAndGbc,
        QuickValue::All,
        QuickValue::Below,
        QuickValue::Above,
        QuickValue::Hours12,
        QuickValue::Hours24,
        QuickValue::KeyA,
        QuickValue::KeyB,
        QuickValue::KeyL,
        QuickValue::KeyR,
        QuickValue::KeySelect,
        QuickValue::KeyStart,
        QuickValue::KeyX,
        QuickValue::KeyY,
        QuickValue::Berry,
        QuickValue::Grape,
        QuickValue::Kiwi,
        QuickValue::Dandelion,
        QuickValue::Teal,
        QuickValue::AtomicPurple,
        QuickValue::Global,
        QuickValue::Custom,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => "On",
            QuickValue::Off => "Off",
            QuickValue::Lcd3x => "LCD3x",
            QuickValue::Grid => "Grid",
            QuickValue::Dot => "Dot",
            QuickValue::Simpletex => "Simpletex",
            QuickValue::GbaOnly => "GBA Only",
            QuickValue::GbAndGbc => "GB / GBC",
            QuickValue::All => "All",
            QuickValue::Below => "Below",
            QuickValue::Above => "Above",
            QuickValue::Hours12 => "12 Hour",
            QuickValue::Hours24 => "24 Hour",
            QuickValue::KeyA => "A",
            QuickValue::KeyB => "B",
            QuickValue::KeyL => "L",
            QuickValue::KeyR => "R",
            QuickValue::KeySelect => "Select",
            QuickValue::KeyStart => "Start",
            QuickValue::KeyX => "Turbo A",
            QuickValue::KeyY => "Turbo B",
            QuickValue::Berry => "Berry",
            QuickValue::Grape => "Grape",
            QuickValue::Kiwi => "Kiwi",
            QuickValue::Dandelion => "Dandelion",
            QuickValue::Teal => "Teal",
            QuickValue::AtomicPurple => "Atomic Purple",
            QuickValue::Global => "Global",
            QuickValue::Custom => "Custom",
        }
    }

    pub fn speed(frames: u8) -> Option<QuickValue> {
        match frames {
            2 => Some(QuickValue::Speed2),
            3 => Some(QuickValue::Speed3),
            4 => Some(QuickValue::Speed4),
            6 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    pub fn key(key: PadKey) -> QuickValue {
        match key {
            PadKey::A => QuickValue::KeyA,
            PadKey::B => QuickValue::KeyB,
            PadKey::L => QuickValue::KeyL,
            PadKey::R => QuickValue::KeyR,
            PadKey::Select => QuickValue::KeySelect,
            PadKey::Start => QuickValue::KeyStart,
            PadKey::X => QuickValue::KeyX,
            PadKey::Y => QuickValue::KeyY,
        }
    }

    pub fn flag(on: bool) -> QuickValue {
        if on {
            QuickValue::On
        } else {
            QuickValue::Off
        }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
pub const QUICK_TOP: f32 = (OUT_H as f32 - QUICK_PITCH * QuickRow::MAIN.len() as f32) / 2.0;
pub const QUICK_EDGE: f32 = 32.0;
const BAR_INSET: f32 = 4.0;
const TYPE_DROP: f32 = 4.0;
const CARET_GAP: f32 = 14.0;
const CARET_PX: f32 = 24.0;
const LEGEND_Y: f32 = 427.0;
pub const QUICK_SPLIT: f32 = LEGEND_Y - 2.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];
const OVER_GAME_ALPHA: f32 = 0.86;

pub fn quick_label_face(row: QuickRow) -> UndoFace {
    quick_text_face(row.label(), MENU_INK)
}

pub fn quick_value_face(text: &str, lit: bool) -> UndoFace {
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK })
}

fn quick_text_face(label: &str, colour: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, label, OUT_W as f32, 1, MENU_PX, MENU_PX);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

pub fn quick_caret_face(right: bool) -> UndoFace {
    let glyph = if right { '\u{f0da}' } else { '\u{f0d9}' };
    let (Some(symbols), Some(label)) = (crate::icon::symbols_font(), text::label_font()) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let (m, cov) = symbols.rasterize(glyph, CARET_PX);
    let (w, h) = (m.width as u32, MENU_H);
    let centre = match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    };
    let top = (centre - m.height as f32 / 2.0).round() as i32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for gy in 0..m.height {
        let y = top + gy as i32;
        if y < 0 || y >= h as i32 {
            continue;
        }
        for gx in 0..m.width {
            let at = ((y as u32 * w + gx as u32) * 4) as usize;
            let a = cov[gy * m.width + gx];
            rgba[at..at + 4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub fn quick_legend_faces() -> [UndoFace; 3] {
    [
        hint_face("B", "Back"),
        arrows_hint_face("Change"),
        hint_face("A", "Open"),
    ]
}

pub struct QuickMenuFaces {
    pub labels: Vec<(TexId, u32, u32)>,
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub carets: [(TexId, u32, u32); 2],
    pub legend: [(TexId, u32); 3],
}

pub struct QuickMenu<'a> {
    pub row: QuickRow,
    pub values: [Option<QuickValue>; QuickRow::ALL.len()],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    pub faces: Option<&'a QuickMenuFaces>,
    pub over_game: bool,
}

impl QuickMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let mut backdrop = opening();
        if self.over_game {
            backdrop[3] = OVER_GAME_ALPHA;
        }
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            colour: backdrop,
        });
        let pitch = pitch(self.row.page().len());
        out.push(Draw::Rect {
            x: 0.0,
            y: row_top(self.row) + BAR_INSET,
            w: OUT_W as f32,
            h: pitch - 2.0 * BAR_INSET,
            colour: edge(),
        });
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (OUT_W as f32 - QUICK_EDGE, MENU_PAD as f32);
        for &row in self.row.page() {
            let y = row_top(row) + TYPE_DROP - (QUICK_PITCH - pitch) / 2.0;
            let lit = row == self.row;
            if let Some(&(tex, w, h)) = faces.labels.get(row.index()) {
                push(out, tex, QUICK_EDGE - pad, y, w, h);
            }
            let value = match row {
                QuickRow::DateTime => self.clock.map(|c| c[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                continue;
            };
            if !lit || row.opens() {
                push(out, tex, right + pad - w as f32, y, w, h);
                continue;
            }
            let [(left_tex, lw, lh), (right_tex, rw, rh)] = faces.carets;
            let rx = right - rw as f32;
            push(out, right_tex, rx, y, rw, rh);
            let vx = rx - CARET_GAP + pad - w as f32;
            push(out, tex, vx, y, w, h);
            push(out, left_tex, vx + pad - CARET_GAP - lw as f32, y, lw, lh);
        }
        let [back, change, open] = faces.legend;
        let other = if self.row.opens() { open } else { change };
        for (tex, w, x) in centred_hints(&[back, other], LEGEND_GAP) {
            push(out, tex, x, LEGEND_Y, w, HINT_H);
        }
    }
}

const TALL_PAGE: usize = 7;
const COMPACT_PITCH: f32 = 42.0;

fn pitch(rows: usize) -> f32 {
    match rows > TALL_PAGE {
        true => COMPACT_PITCH,
        false => QUICK_PITCH,
    }
}

pub fn row_top(row: QuickRow) -> f32 {
    let rows = row.page().len();
    let (pitch, room) = match rows > TALL_PAGE {
        true => (COMPACT_PITCH, LEGEND_Y),
        false => (QUICK_PITCH, OUT_H as f32),
    };
    (room - pitch * rows as f32) / 2.0 + pitch * row.position() as f32
}

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}
