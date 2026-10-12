mod common;

use common::{app_playing_in, await_speed, pad_of, session_playing, tmp_root_with_carts};
use slot::app::{App, Phase};
use slot::emu::Speed;
use slot_input::{Action, Btn, RawEvent};
use slot_retro::ButtonMask;
use slot_store::{
    read_game_layout, read_slot_state, write_slot_state, Layout, PadKey, Platform, Shader,
    SlotState,
};
use slot_ui::{QuickRow, QuickValue, SlotMode};
use tempfile::TempDir;

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn paused_game() -> (TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::QuickMenu);
    assert_eq!(a.paused(), Some(QuickRow::Resume));
    (d, a)
}

fn go_to(a: &mut App, row: QuickRow) {
    for _ in 0..row.position() {
        press(a, Btn::Down);
    }
    assert_eq!(a.paused(), Some(row));
}

fn swapped() -> Layout {
    let mut l = Layout::default();
    l.set(PadKey::A, PadKey::B);
    l.set(PadKey::B, PadKey::A);
    l
}

#[test]
fn the_pause_menu_is_closed_until_menu_is_tapped_in_game() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let a = app_playing_in(d.path(), "Emerald");
    assert_eq!(a.paused(), None);
}

#[test]
fn menu_and_b_close_the_pause_menu_back_onto_the_game() {
    for close in [Action::QuickMenu, Action::GbaDown(Btn::B)] {
        let (_d, mut a) = paused_game();
        a.apply(close);
        assert_eq!(a.paused(), None, "{close:?}");
        assert!(matches!(a.phase(), Phase::Playing { .. }), "{close:?}");
    }
}

#[test]
fn it_reopens_on_resume_every_time() {
    let (_d, mut a) = paused_game();
    press(&mut a, Btn::Down);
    press(&mut a, Btn::Down);
    a.apply(Action::QuickMenu);
    a.apply(Action::QuickMenu);
    assert_eq!(a.paused(), Some(QuickRow::Resume));
}

#[test]
fn up_and_down_walk_the_pause_rows_and_wrap_at_both_ends() {
    let (_d, mut a) = paused_game();
    press(&mut a, Btn::Up);
    assert_eq!(a.paused(), Some(QuickRow::EjectGame));
    for want in [
        QuickRow::Resume,
        QuickRow::SaveState,
        QuickRow::LoadState,
        QuickRow::Shader,
        QuickRow::GameControls,
        QuickRow::EjectGame,
    ] {
        press(&mut a, Btn::Down);
        assert_eq!(a.paused(), Some(want));
    }
}

#[test]
fn a_on_resume_closes_the_menu() {
    let (_d, mut a) = paused_game();
    press(&mut a, Btn::A);
    assert_eq!(a.paused(), None);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn a_on_eject_game_ejects_the_cart() {
    let (_d, mut a) = paused_game();
    go_to(&mut a, QuickRow::EjectGame);
    press(&mut a, Btn::A);
    assert_eq!(a.paused(), None);
    assert!(
        matches!(a.phase(), Phase::Ejecting { .. }),
        "{:?}",
        a.phase()
    );
}

#[test]
fn the_pause_menu_is_refused_while_a_link_session_is_live() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.begin_link(0);
    a.apply(Action::QuickMenu);
    assert_eq!(a.paused(), None);
    assert!(a.refusal_active(a.now()), "the refusal never shook");
}

#[test]
fn the_pause_menu_swallows_the_game_buttons() {
    let (_d, mut a) = paused_game();
    a.apply(Action::Insert);
    a.apply(Action::GbaDown(Btn::X));
    assert_eq!(a.paused(), Some(QuickRow::Resume));
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn save_state_opens_the_picker_in_save_mode_and_a_saves_into_the_chosen_slot() {
    let (d, mut a) = paused_game();
    go_to(&mut a, QuickRow::SaveState);
    press(&mut a, Btn::A);
    assert_eq!(a.paused(), None);
    assert_eq!(a.slot_mode(), Some(SlotMode::Save));
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    press(&mut a, Btn::Right);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.paused(), None, "saving did not hand the game back");
    let slots =
        slot_store::StateRing::new(d.path(), Platform::Gba, slot_store::Core::Mgba, "Emerald")
            .slots();
    assert!(slots[0].is_none() && slots[1].is_some() && slots[2].is_none());
    assert_eq!(a.toast(), Some(slot_ui::Toast::SlotSaved(1)));
}

#[test]
fn load_state_opens_the_picker_in_load_mode_and_b_goes_back_to_the_pause_menu() {
    let (_d, mut a) = paused_game();
    go_to(&mut a, QuickRow::LoadState);
    press(&mut a, Btn::A);
    assert_eq!(a.slot_mode(), Some(SlotMode::Load));
    press(&mut a, Btn::B);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.paused(), Some(QuickRow::LoadState));
    assert!(!a.slots_open());
}

#[test]
fn save_mode_b_goes_back_to_the_save_row_without_saving() {
    let (d, mut a) = paused_game();
    go_to(&mut a, QuickRow::SaveState);
    press(&mut a, Btn::A);
    press(&mut a, Btn::B);
    assert_eq!(a.paused(), Some(QuickRow::SaveState));
    let ring =
        slot_store::StateRing::new(d.path(), Platform::Gba, slot_store::Core::Mgba, "Emerald");
    assert!(ring.slots().iter().all(Option::is_none));
}

#[test]
fn loading_an_empty_slot_from_the_pause_menu_keeps_the_picker_open() {
    let (_d, mut a) = paused_game();
    go_to(&mut a, QuickRow::LoadState);
    press(&mut a, Btn::A);
    press(&mut a, Btn::A);
    assert!(a.slots_open(), "an empty slot closed the picker");
    assert!(a.refusal_active(a.now()));
}

#[test]
fn left_and_right_on_shader_step_the_screen_shader_both_ways() {
    let (d, mut a) = paused_game();
    go_to(&mut a, QuickRow::Shader);
    let start = a.screen_shader();
    assert!(a.quick_value(QuickRow::Shader).is_some());
    press(&mut a, Btn::Right);
    let next = a.screen_shader();
    assert_eq!(next, start.step(&Shader::GBA, true));
    assert_ne!(next, start);
    assert_eq!(read_slot_state(d.path()).shader_gba, next);
    assert_eq!(a.paused(), Some(QuickRow::Shader), "the menu moved");
    press(&mut a, Btn::Left);
    assert_eq!(a.screen_shader(), start);
}

#[test]
fn game_controls_opens_its_page_and_b_backs_out_one_level_at_a_time() {
    let (_d, mut a) = paused_game();
    go_to(&mut a, QuickRow::GameControls);
    press(&mut a, Btn::A);
    assert_eq!(a.paused(), Some(QuickRow::GameLayout));
    press(&mut a, Btn::Down);
    assert_eq!(a.paused(), Some(QuickRow::GameRemapA));
    press(&mut a, Btn::B);
    assert_eq!(a.paused(), Some(QuickRow::GameControls));
    press(&mut a, Btn::B);
    assert_eq!(a.paused(), None);
}

#[test]
fn a_game_starts_on_the_global_layout() {
    let (_d, mut a) = paused_game();
    go_to(&mut a, QuickRow::GameControls);
    press(&mut a, Btn::A);
    assert_eq!(
        a.quick_value(QuickRow::GameLayout),
        Some(QuickValue::Global)
    );
}

#[test]
fn custom_copies_the_global_layout_into_a_file_for_this_game() {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Emerald".into()),
            clock_set: true,
            layout: swapped(),
            ..SlotState::default()
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_snapshot(common::StubSnapshot::boxed());
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::QuickMenu);
    go_to(&mut a, QuickRow::GameControls);
    press(&mut a, Btn::A);
    press(&mut a, Btn::Right);
    assert_eq!(
        a.quick_value(QuickRow::GameLayout),
        Some(QuickValue::Custom)
    );
    assert_eq!(
        read_game_layout(d.path(), Platform::Gba, "Emerald"),
        Some(swapped())
    );
    assert_eq!(a.layout(), swapped());
}

#[test]
fn toggling_back_to_global_removes_the_games_file() {
    let (d, mut a) = paused_game();
    go_to(&mut a, QuickRow::GameControls);
    press(&mut a, Btn::A);
    press(&mut a, Btn::Right);
    assert!(read_game_layout(d.path(), Platform::Gba, "Emerald").is_some());
    press(&mut a, Btn::Left);
    assert_eq!(read_game_layout(d.path(), Platform::Gba, "Emerald"), None);
    assert_eq!(
        a.quick_value(QuickRow::GameLayout),
        Some(QuickValue::Global)
    );
}

#[test]
fn editing_a_game_remap_switches_the_game_to_custom_and_writes_only_its_file() {
    let (d, mut a) = paused_game();
    go_to(&mut a, QuickRow::GameControls);
    press(&mut a, Btn::A);
    press(&mut a, Btn::Down);
    press(&mut a, Btn::Right);
    assert_eq!(
        a.quick_value(QuickRow::GameLayout),
        Some(QuickValue::Custom)
    );
    assert_eq!(a.quick_value(QuickRow::GameRemapA), Some(QuickValue::KeyB));
    let file = read_game_layout(d.path(), Platform::Gba, "Emerald").expect("no game file");
    assert_eq!(file.sends(PadKey::A), PadKey::B);
    assert_eq!(file.sends(PadKey::B), PadKey::B);
    assert_eq!(
        read_slot_state(d.path()).layout,
        Layout::default(),
        "the global layout moved with the game's"
    );
    assert_eq!(
        read_game_layout(d.path(), Platform::Gba, "Fusion"),
        None,
        "another game picked it up"
    );
}

#[test]
fn a_game_layout_does_not_outlive_the_cart_in_the_app() {
    let (_d, mut a) = paused_game();
    go_to(&mut a, QuickRow::GameControls);
    press(&mut a, Btn::A);
    press(&mut a, Btn::Down);
    press(&mut a, Btn::Right);
    assert_ne!(a.layout(), Layout::default());
    a.apply(Action::Eject);
    assert_eq!(a.layout(), Layout::default());
}

#[test]
fn while_paused_the_core_is_held_still_and_resume_lets_it_go() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut s = session_playing(d.path());
    await_speed(&mut s, Speed::Normal);
    s.app_mut().apply(Action::QuickMenu);
    assert!(s.app().paused().is_some());
    await_speed(&mut s, Speed::Paused);
    s.app_mut().apply(Action::GbaDown(Btn::A));
    assert_eq!(s.app().paused(), None);
    await_speed(&mut s, Speed::Normal);
}

#[test]
fn opening_the_pause_menu_lets_go_of_every_held_button() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut s = session_playing(d.path());
    s.feed([RawEvent::Down(Btn::A)], 100);
    assert_ne!(pad_of(&s) & ButtonMask::A, 0, "A never reached the game");
    s.feed([RawEvent::Down(Btn::Menu)], 120);
    s.feed([RawEvent::Up(Btn::Menu)], 150);
    assert!(s.app().paused().is_some(), "a tap of MENU did not pause");
    assert_eq!(pad_of(&s), 0, "the pad stayed held under the menu");
    s.feed([RawEvent::Down(Btn::B)], 140);
    assert_eq!(pad_of(&s), 0, "the game heard a button under the menu");
}

#[test]
fn a_double_tapped_menu_opens_load_and_b_goes_straight_back_to_the_game() {
    let (_d, mut a) = paused_game();
    a.apply(Action::Polaroids);
    assert_eq!(a.slot_mode(), Some(SlotMode::Load));
    a.apply(Action::GbaDown(Btn::B));
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(
        a.paused(),
        None,
        "the double tap left the pause menu under the picker"
    );
}
