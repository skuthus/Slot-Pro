use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::gb_palette::GbPalette;
use crate::layout::Layout;
use crate::platform::Platform;

pub const BRIGHTNESS_MAX: u8 = 9;
pub const BLUE_LIGHT_MAX: u8 = 9;
pub const VOLUME_MAX: u8 = 100;

pub const UTC_OFFSET_MIN: i16 = -720;
pub const UTC_OFFSET_MAX: i16 = 840;

pub const FF_SPEEDS: [u8; 4] = [2, 3, 4, 6];

pub const FF_SPEED_DEFAULT: u8 = 6;

pub const RUN_AHEAD_MAX: u8 = 2;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SlotState {
    pub cart: Option<String>,
    pub cart_platform: Option<Platform>,
    pub brightness: u8,
    pub blue_light: u8,
    pub volume: u8,
    pub volume_hp: u8,
    pub muted: bool,
    pub muted_hp: bool,
    pub clock_set: bool,
    pub utc_offset_min: i16,
    pub rumble: bool,
    pub ff_speed: u8,
    pub ff_sound: bool,
    pub colour_correction: ColourCorrection,
    pub shader_gba: Shader,
    pub shader_gb: Shader,
    pub eject_save: bool,
    pub turbo: bool,
    pub rewind: bool,
    pub gb_palettes: bool,
    pub gb_palette: GbPalette,
    pub clock_24h: bool,
    pub cart_title: CartTitle,
    pub layout: Layout,
    pub gb_frame: GbFrame,
    pub analog: bool,
    pub start_favorites: bool,
    pub run_ahead: u8,
    pub lcd_ghosting: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GbFrame {
    Off,
    Berry,
    Grape,
    Kiwi,
    Dandelion,
    Teal,
    AtomicPurple,
}

impl GbFrame {
    pub const ALL: [GbFrame; 7] = [
        GbFrame::Off,
        GbFrame::Berry,
        GbFrame::Grape,
        GbFrame::Kiwi,
        GbFrame::Dandelion,
        GbFrame::Teal,
        GbFrame::AtomicPurple,
    ];

    pub fn name(self) -> &'static str {
        match self {
            GbFrame::Off => "off",
            GbFrame::Berry => "berry",
            GbFrame::Grape => "grape",
            GbFrame::Kiwi => "kiwi",
            GbFrame::Dandelion => "dandelion",
            GbFrame::Teal => "teal",
            GbFrame::AtomicPurple => "atomic_purple",
        }
    }

    pub fn parse(name: &str) -> Option<GbFrame> {
        Self::ALL.into_iter().find(|f| f.name() == name)
    }

    pub fn step(self, right: bool) -> GbFrame {
        step_in(&Self::ALL, self, right)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ColourCorrection {
    Off,
    GbaOnly,
    GbAndGbc,
    All,
}

impl ColourCorrection {
    pub const ALL: [ColourCorrection; 4] = [
        ColourCorrection::Off,
        ColourCorrection::GbaOnly,
        ColourCorrection::GbAndGbc,
        ColourCorrection::All,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ColourCorrection::Off => "off",
            ColourCorrection::GbaOnly => "gba",
            ColourCorrection::GbAndGbc => "gb_gbc",
            ColourCorrection::All => "all",
        }
    }

    pub fn parse(name: &str) -> Option<ColourCorrection> {
        match name {
            "0" => Some(ColourCorrection::Off),
            "1" | "on" => Some(ColourCorrection::All),
            _ => Self::ALL.into_iter().find(|c| c.name() == name),
        }
    }

    pub fn applies_to(self, platform: Platform) -> bool {
        match self {
            ColourCorrection::Off => false,
            ColourCorrection::GbaOnly => platform == Platform::Gba,
            ColourCorrection::GbAndGbc => platform != Platform::Gba,
            ColourCorrection::All => true,
        }
    }

    pub fn step(self, right: bool) -> ColourCorrection {
        step_in(&Self::ALL, self, right)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CartTitle {
    Off,
    Below,
    Above,
}

impl CartTitle {
    pub const ALL: [CartTitle; 3] = [CartTitle::Off, CartTitle::Below, CartTitle::Above];

    pub fn name(self) -> &'static str {
        match self {
            CartTitle::Off => "off",
            CartTitle::Below => "below",
            CartTitle::Above => "above",
        }
    }

    pub fn parse(name: &str) -> Option<CartTitle> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }

    pub fn step(self, right: bool) -> CartTitle {
        step_in(&Self::ALL, self, right)
    }
}

fn step_in<T: Copy + PartialEq>(choices: &[T], at: T, right: bool) -> T {
    let n = choices.len();
    let i = choices.iter().position(|&c| c == at).unwrap_or(0);
    choices[if right { (i + 1) % n } else { (i + n - 1) % n }]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Shader {
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    Gbc,
}

impl Shader {
    pub const GBA: [Shader; 4] = [Shader::Off, Shader::Lcd3x, Shader::Grid, Shader::Dot];
    pub const GB: [Shader; 4] = [Shader::Off, Shader::Grid, Shader::Simpletex, Shader::Gbc];

    pub fn name(self) -> &'static str {
        match self {
            Shader::Off => "off",
            Shader::Lcd3x => "lcd3x",
            Shader::Grid => "grid",
            Shader::Dot => "dot",
            Shader::Simpletex => "simpletex",
            Shader::Gbc => "gbc",
        }
    }

    pub fn parse(name: &str) -> Option<Shader> {
        [
            Shader::Off,
            Shader::Lcd3x,
            Shader::Grid,
            Shader::Dot,
            Shader::Simpletex,
            Shader::Gbc,
        ]
        .into_iter()
        .find(|s| s.name() == name)
    }

    pub fn step(self, choices: &[Shader], right: bool) -> Shader {
        step_in(choices, self, right)
    }
}

impl Default for SlotState {
    fn default() -> Self {
        SlotState {
            cart: None,
            cart_platform: None,
            brightness: 5,
            blue_light: 0,
            volume: 60,
            volume_hp: 60,
            muted: false,
            muted_hp: false,
            clock_set: false,
            utc_offset_min: 0,
            rumble: true,
            ff_speed: FF_SPEED_DEFAULT,
            ff_sound: false,
            colour_correction: ColourCorrection::Off,
            shader_gba: Shader::Grid,
            shader_gb: Shader::Simpletex,
            eject_save: true,
            turbo: true,
            rewind: true,
            gb_palettes: false,
            gb_palette: GbPalette::DEFAULT,
            clock_24h: true,
            cart_title: CartTitle::Off,
            layout: Layout::default(),
            gb_frame: GbFrame::Off,
            analog: false,
            start_favorites: false,
            run_ahead: 0,
            lcd_ghosting: true,
        }
    }
}

fn state_path(root: &Path) -> PathBuf {
    root.join("Config").join("slot.state")
}

pub fn read_slot_state(root: &Path) -> SlotState {
    std::fs::read(state_path(root))
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| parse(&s))
        .unwrap_or_default()
}

pub fn write_slot_state(root: &Path, s: &SlotState) -> std::io::Result<()> {
    let text = format!(
        "cart={}\ncart_platform={}\nbrightness={}\nblue_light={}\nvolume={}\nvolume_hp={}\nmuted={}\nmuted_hp={}\nclock_set={}\nutc_offset_min={}\nrumble={}\nff_speed={}\nff_sound={}\ncolour_correction={}\nshader_gba={}\nshader_gb={}\neject_save={}\nturbo={}\nrewind={}\ngb_palettes={}\ngb_palette={}\nclock_24h={}\ncart_title={}\nlayout={}\ngb_frame={}\nanalog={}\nstart_favorites={}\nrun_ahead={}\nlcd_ghosting={}\n",
        s.cart.as_deref().unwrap_or(""),
        s.cart_platform.map_or(String::new(), platform_key),
        s.brightness,
        s.blue_light,
        s.volume,
        s.volume_hp,
        s.muted as u8,
        s.muted_hp as u8,
        s.clock_set as u8,
        s.utc_offset_min,
        s.rumble as u8,
        s.ff_speed,
        s.ff_sound as u8,
        s.colour_correction.name(),
        s.shader_gba.name(),
        s.shader_gb.name(),
        s.eject_save as u8,
        s.turbo as u8,
        s.rewind as u8,
        s.gb_palettes as u8,
        s.gb_palette.core_name(),
        s.clock_24h as u8,
        s.cart_title.name(),
        s.layout.name(),
        s.gb_frame.name(),
        s.analog as u8,
        s.start_favorites as u8,
        s.run_ahead,
        s.lcd_ghosting as u8
    );
    atomic_write(&state_path(root), text.as_bytes())
}

fn parse(text: &str) -> Option<SlotState> {
    let mut cart = None;
    let mut cart_platform = None;
    let mut brightness = None;
    let mut blue_light = None;
    let mut volume = None;
    let mut volume_hp = None;
    let mut muted = None;
    let mut muted_hp = None;
    let mut clock_set = None;
    let mut utc_offset_min = None;
    let mut rumble = None;
    let mut ff_speed = None;
    let mut ff_sound = None;
    let mut colour_correction = None;
    let mut shader_gba = None;
    let mut shader_gb = None;
    let mut eject_save = None;
    let mut turbo = None;
    let mut rewind = None;
    let mut gb_palettes = None;
    let mut gb_palette = None;
    let mut clock_24h = None;
    let mut cart_title = None;
    let mut layout = None;
    let mut gb_frame = None;
    let mut analog = None;
    let mut start_favorites = None;
    let mut run_ahead = None;
    let mut lcd_ghosting = None;
    for line in text.lines().filter(|l| !l.is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "cart" => cart = Some(value.to_string()),
            "cart_platform" => cart_platform = platform_value(value),
            "brightness" => brightness = Some(level(value, BRIGHTNESS_MAX)?),
            "blue_light" => blue_light = Some(level(value, BLUE_LIGHT_MAX)?),
            "volume" => volume = Some(level(value, VOLUME_MAX)?),
            "volume_hp" => volume_hp = Some(level(value, VOLUME_MAX)?),
            "muted" => muted = Some(level(value, 1)? == 1),
            "muted_hp" => muted_hp = Some(level(value, 1)? == 1),
            "clock_set" => clock_set = Some(level(value, 1)? == 1),
            "utc_offset_min" => utc_offset_min = Some(offset(value)?),
            "rumble" => rumble = flag(value),
            "ff_speed" => ff_speed = ff_speed_value(value),
            "ff_sound" => ff_sound = flag(value),
            "colour_correction" => colour_correction = ColourCorrection::parse(value),
            "shader_gba" => shader_gba = Shader::parse(value).filter(|s| Shader::GBA.contains(s)),
            "eject_save" => eject_save = flag(value),
            "turbo" => turbo = flag(value),
            "rewind" => rewind = flag(value),
            "gb_palettes" => gb_palettes = flag(value),
            "gb_palette" => gb_palette = GbPalette::parse(value),
            "clock_24h" => clock_24h = flag(value),
            "cart_title" => cart_title = CartTitle::parse(value),
            "layout" => layout = Layout::parse(value),
            "gb_frame" => gb_frame = GbFrame::parse(value),
            "analog" => analog = flag(value),
            "start_favorites" => start_favorites = flag(value),
            "run_ahead" => run_ahead = level(value, RUN_AHEAD_MAX),
            "lcd_ghosting" => lcd_ghosting = flag(value),
            "shader_gb" => shader_gb = Shader::parse(value).filter(|s| Shader::GB.contains(s)),
            _ => {}
        }
    }
    let cart = cart?;
    let fallback = SlotState::default();
    Some(SlotState {
        cart: (!cart.is_empty()).then_some(cart),
        cart_platform,
        brightness: brightness?,
        blue_light: blue_light?,
        volume: volume?,
        volume_hp: volume_hp.or(volume)?,
        muted: muted?,
        muted_hp: muted_hp.or(muted)?,
        clock_set: clock_set?,
        utc_offset_min: utc_offset_min?,
        rumble: rumble.unwrap_or(fallback.rumble),
        ff_speed: ff_speed.unwrap_or(fallback.ff_speed),
        ff_sound: ff_sound.unwrap_or(fallback.ff_sound),
        colour_correction: colour_correction.unwrap_or(fallback.colour_correction),
        shader_gba: shader_gba.unwrap_or(fallback.shader_gba),
        shader_gb: shader_gb.unwrap_or(fallback.shader_gb),
        eject_save: eject_save.unwrap_or(fallback.eject_save),
        turbo: turbo.unwrap_or(fallback.turbo),
        rewind: rewind.unwrap_or(fallback.rewind),
        gb_palettes: gb_palettes.unwrap_or(fallback.gb_palettes),
        gb_palette: gb_palette.unwrap_or(fallback.gb_palette),
        clock_24h: clock_24h.unwrap_or(fallback.clock_24h),
        cart_title: cart_title.unwrap_or(fallback.cart_title),
        layout: layout.unwrap_or(fallback.layout),
        gb_frame: gb_frame.unwrap_or(fallback.gb_frame),
        analog: analog.unwrap_or(fallback.analog),
        start_favorites: start_favorites.unwrap_or(fallback.start_favorites),
        run_ahead: run_ahead.unwrap_or(fallback.run_ahead),
        lcd_ghosting: lcd_ghosting.unwrap_or(fallback.lcd_ghosting),
    })
}

fn platform_key(platform: Platform) -> String {
    platform.dir_name().to_ascii_lowercase()
}

fn platform_value(value: &str) -> Option<Platform> {
    Platform::ALL
        .into_iter()
        .find(|p| value.eq_ignore_ascii_case(p.dir_name()))
}

fn offset(value: &str) -> Option<i16> {
    value
        .parse()
        .ok()
        .filter(|n| (UTC_OFFSET_MIN..=UTC_OFFSET_MAX).contains(n))
}

fn ff_speed_value(value: &str) -> Option<u8> {
    value.parse().ok().filter(|n| FF_SPEEDS.contains(n))
}

fn level(value: &str, max: u8) -> Option<u8> {
    value.parse().ok().filter(|n| *n <= max)
}

fn flag(value: &str) -> Option<bool> {
    level(value, 1).map(|n| n == 1)
}
