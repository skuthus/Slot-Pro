mod common;

use common::{pad_of, session_playing, tmp_root_with_carts};
use slot::session::Session;
use slot_input::{Btn, RawEvent};
use slot_retro::ButtonMask;
use slot_store::{write_game_layout, write_slot_state, Layout, PadKey, Platform, SlotState};
use tempfile::TempDir;

fn layout(maps: &[(PadKey, PadKey)]) -> Layout {
    let mut l = Layout::default();
    for (pressed, sends) in maps {
        l.set(*pressed, *sends);
    }
    l
}

fn session_with(global: Layout, game: Option<Layout>) -> (TempDir, Session) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            layout: global,
            ..SlotState::default()
        },
    )
    .unwrap();
    if game.is_some() {
        write_game_layout(d.path(), Platform::Gba, "Emerald", game).unwrap();
    }
    let s = session_playing(d.path());
    (d, s)
}

fn press(s: &mut Session, btn: Btn, now: &mut u64) -> u16 {
    *now += 20;
    s.feed([RawEvent::Down(btn)], *now);
    pad_of(s)
}

fn release(s: &mut Session, btn: Btn, now: &mut u64) -> u16 {
    *now += 20;
    s.feed([RawEvent::Up(btn)], *now);
    pad_of(s)
}

#[test]
fn with_the_default_layout_every_button_reaches_the_core_as_itself() {
    let (_d, mut s) = session_with(Layout::default(), None);
    let mut now = 1_000;
    for (btn, bit) in [
        (Btn::A, ButtonMask::A),
        (Btn::B, ButtonMask::B),
        (Btn::Start, ButtonMask::START),
    ] {
        assert_eq!(press(&mut s, btn, &mut now), bit, "{btn:?}");
        assert_eq!(release(&mut s, btn, &mut now), 0, "{btn:?}");
    }
}

#[test]
fn pressing_a_with_a_sent_as_b_sets_the_b_bit_and_not_the_a_bit() {
    let (_d, mut s) = session_with(layout(&[(PadKey::A, PadKey::B)]), None);
    let mut now = 1_000;
    let held = press(&mut s, Btn::A, &mut now);
    assert_eq!(
        held & ButtonMask::B,
        ButtonMask::B,
        "B never reached the core"
    );
    assert_eq!(held & ButtonMask::A, 0, "A reached the core as well");
    assert_eq!(release(&mut s, Btn::A, &mut now), 0, "B stuck down");
}

#[test]
fn a_swap_sends_each_button_to_the_others_place() {
    let (_d, mut s) = session_with(
        layout(&[(PadKey::A, PadKey::B), (PadKey::B, PadKey::A)]),
        None,
    );
    let mut now = 1_000;
    assert_eq!(press(&mut s, Btn::A, &mut now), ButtonMask::B);
    assert_eq!(release(&mut s, Btn::A, &mut now), 0);
    assert_eq!(press(&mut s, Btn::B, &mut now), ButtonMask::A);
    assert_eq!(release(&mut s, Btn::B, &mut now), 0);
}

#[test]
fn shoulders_and_start_remap_too() {
    let (_d, mut s) = session_with(
        layout(&[(PadKey::L, PadKey::R), (PadKey::Start, PadKey::Select)]),
        None,
    );
    let mut now = 1_000;
    assert_eq!(press(&mut s, Btn::L1, &mut now), ButtonMask::R);
    assert_eq!(release(&mut s, Btn::L1, &mut now), 0);
    assert_eq!(press(&mut s, Btn::Start, &mut now), ButtonMask::SELECT);
    assert_eq!(release(&mut s, Btn::Start, &mut now), 0);
}

#[test]
fn the_directions_are_never_remapped() {
    let (_d, mut s) = session_with(layout(&[(PadKey::A, PadKey::Start)]), None);
    let mut now = 1_000;
    assert_eq!(press(&mut s, Btn::Up, &mut now), ButtonMask::UP);
    assert_eq!(release(&mut s, Btn::Up, &mut now), 0);
}

#[test]
fn a_games_own_layout_wins_over_the_global_one() {
    let (_d, mut s) = session_with(
        layout(&[(PadKey::A, PadKey::Start)]),
        Some(layout(&[(PadKey::A, PadKey::B)])),
    );
    let mut now = 1_000;
    assert_eq!(press(&mut s, Btn::A, &mut now), ButtonMask::B);
}

#[test]
fn a_game_without_a_layout_file_uses_the_global_one() {
    let (_d, mut s) = session_with(layout(&[(PadKey::A, PadKey::Start)]), None);
    let mut now = 1_000;
    assert_eq!(press(&mut s, Btn::A, &mut now), ButtonMask::START);
}

#[test]
fn another_games_layout_file_is_not_applied() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_game_layout(
        d.path(),
        Platform::Gba,
        "Fusion",
        Some(layout(&[(PadKey::A, PadKey::B)])),
    )
    .unwrap();
    let mut s = session_playing(d.path());
    let mut now = 1_000;
    assert_eq!(press(&mut s, Btn::A, &mut now), ButtonMask::A);
}
