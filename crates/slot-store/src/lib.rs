mod atomic;
pub mod cart_shell;
mod config;
mod core;
mod favorites;
pub mod gb;
mod gb_palette;
mod gba;
pub mod ini;
mod layout;
mod platform;
mod ring;
mod scan;
mod slot_state;
mod stamp;
mod theme;

pub use atomic::atomic_write;
pub use cart_shell::{Outline, ShellChoice, ShellFinish, CART_SHELL_FILE, LABELS_SHELL_FILE};
pub use config::{move_config, CONFIG_DIR};
pub use core::{
    core_for, core_for_platform, default_for_game, read_selected_cores, sgb_cart,
    write_selected_core, Core, SELECTED_CORE_FILE,
};
pub use favorites::{read_favorites, write_favorites};
pub use gb_palette::{
    curated_palette, gb_header_title, read_game_palette, write_game_palette, GbPalette,
};
pub use gba::{header_clean, header_code, header_title, header_title_code};
pub use layout::{read_game_layout, write_game_layout, Layout, PadKey};
pub use platform::Platform;
pub use ring::{StateEntry, StateRing, RING_MAX, SLOT_COUNT};
pub use scan::{initial, is_hidden, listing, scan, sort_key, Cart, StoreError};
pub use slot_state::{
    read_slot_state, write_slot_state, CartTitle, ColourCorrection, GbFrame, Shader, SlotState,
    BLUE_LIGHT_MAX, BRIGHTNESS_MAX, FF_SPEEDS, FF_SPEED_DEFAULT, RUN_AHEAD_MAX, UTC_OFFSET_MAX,
    UTC_OFFSET_MIN, VOLUME_MAX,
};
pub use stamp::{
    civil_from_days, days_from_civil, days_in_month, format_stamp, parse_stamp, stamp_now,
};
pub use theme::{Theme, THEME_FILE};
