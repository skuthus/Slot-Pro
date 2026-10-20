mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use common::{repo_root, session_with_platform, tmp_root_with_carts, tmp_root_with_gb_carts};
use slot::app::Phase;
use slot::session::Session;
use slot_input::{Action, Btn, Millis, RawEvent};
use slot_store::{
    read_game_palette, write_game_palette, write_slot_state, GbPalette, Platform, SlotState,
};
use slot_ui::Toast;
use tempfile::TempDir;

const FRAME_MS: Millis = 16;
const DT: f32 = 1.0 / 60.0;

fn step(s: &mut Session, now: &mut Millis) {
    *now += FRAME_MS;
    s.feed([], *now);
    s.update(DT);
}

fn event(s: &mut Session, ev: RawEvent, now: &mut Millis) {
    *now += FRAME_MS;
    s.feed([ev], *now);
    s.update(DT);
}

fn play(s: &mut Session, now: &mut Millis) {
    event(s, RawEvent::Down(Btn::A), now);
    event(s, RawEvent::Up(Btn::A), now);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        step(s, now);
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn catrap_with_colour_flag(cgb: u8, palettes: bool) -> Option<TempDir> {
    let real = repo_root().join("sdcard/Games/GB/Catrap (USA).gb");
    let mut rom = std::fs::read(&real).ok()?;
    rom[0x143] = cgb;
    let d = tmp_root_with_gb_carts(&["Catrap"]);
    std::fs::write(d.path().join("Games/GB/Catrap.gb"), rom).expect("write rom");
    palettes_on(d.path(), palettes);
    Some(d)
}

fn palettes_on(root: &Path, on: bool) {
    write_slot_state(
        root,
        &SlotState {
            gb_colors: on,
            ..SlotState::default()
        },
    )
    .expect("write slot.state");
}

fn kept(root: &Path) -> Option<GbPalette> {
    read_game_palette(root, Platform::Gb, "Catrap")
}

#[test]
fn select_r2_moves_a_game_off_auto_onto_the_first_favourite_and_keeps_it_for_that_game() {
    let Some(d) = catrap_with_colour_flag(0x00, true) else {
        eprintln!("no Catrap on this machine's card, skipping");
        return;
    };
    let (mut s, _motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    let palette = s.app().game_palette().expect("the cart is Game Boy only");
    assert_eq!(palette.chosen, None, "a fresh game starts on Auto");
    assert_eq!(
        palette.shown(),
        GbPalette::parse("AYY4"),
        "Auto shows slot's pick for Catrap"
    );

    s.app_mut().apply(Action::PaletteNext);
    let first = GbPalette::in_cycle_order().next();
    assert_eq!(s.app().game_palette().and_then(|p| p.chosen), first);
    assert_eq!(s.app().toast(), first.map(Toast::Palette));
    assert_eq!(kept(d.path()), first, "the pick was not saved for the game");
}

#[test]
fn select_l2_from_auto_wraps_to_the_last_palette_and_back_to_auto() {
    let Some(d) = catrap_with_colour_flag(0x00, true) else {
        eprintln!("no Catrap on this machine's card, skipping");
        return;
    };
    let (mut s, _motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    s.app_mut().apply(Action::PalettePrev);
    let last = GbPalette::in_cycle_order().last();
    assert_eq!(s.app().game_palette().and_then(|p| p.chosen), last);
    assert_eq!(kept(d.path()), last);

    s.app_mut().apply(Action::PaletteNext);
    assert_eq!(s.app().game_palette().and_then(|p| p.chosen), None);
    assert_eq!(s.app().toast(), Some(Toast::PaletteAuto));
    assert_eq!(kept(d.path()), None, "Auto left a file behind");
}

#[test]
fn select_r2_does_nothing_with_colours_off_or_in_a_colour_game() {
    for (cgb, on) in [(0x00, false), (0x80, true)] {
        let Some(d) = catrap_with_colour_flag(cgb, on) else {
            eprintln!("no Catrap on this machine's card, skipping");
            return;
        };
        let (mut s, _motor) = session_with_platform(d.path());
        let mut now = 0;
        play(&mut s, &mut now);
        s.app_mut().apply(Action::PaletteNext);
        assert_eq!(
            s.app().toast(),
            None,
            "cgb={cgb:#x} on={on}: a palette toast showed"
        );
        assert_eq!(kept(d.path()), None, "cgb={cgb:#x} on={on}");
    }
}

#[test]
fn colours_off_shows_the_dmg_green_whatever_the_game_chose() {
    let Some(d) = catrap_with_colour_flag(0x00, false) else {
        eprintln!("no Catrap on this machine's card, skipping");
        return;
    };
    write_game_palette(d.path(), Platform::Gb, "Catrap", GbPalette::parse("Hollow")).unwrap();
    let (mut s, _motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    assert_eq!(
        s.app().game_palette().and_then(|p| p.shown()),
        Some(GbPalette::DEFAULT)
    );
}

#[test]
fn select_r2_on_the_carousel_does_nothing() {
    let d = tmp_root_with_carts(&["Emerald"]);
    palettes_on(d.path(), true);
    let (mut s, _motor) = session_with_platform(d.path());
    s.app_mut().apply(Action::PaletteNext);
    assert_eq!(s.app().toast(), None);
}
