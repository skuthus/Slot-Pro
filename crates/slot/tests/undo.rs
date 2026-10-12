mod common;

use common::{app_playing_in, app_playing_with, tmp_root_with_carts, CoreSnapshot};
use slot_input::{Action, Btn};
use slot_store::{Core, Platform, StateRing};

#[test]
fn undoing_a_save_restores_what_the_slot_held_before() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.write_slot(0, &[7u8; 64], b"old-png", "2026-08-09_00-00-00")
        .unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    assert_eq!(r.read_slot(0).unwrap().1, [9u8; 1024]);

    a.undo(2_000);
    let (stamp, state, thumb) = r.read_slot(0).expect("the slot was emptied");
    assert_eq!(stamp, "2026-08-09_00-00-00");
    assert_eq!(state, [7u8; 64]);
    assert_eq!(thumb, b"old-png");
}

#[test]
fn undoing_a_save_into_an_empty_slot_empties_it_again() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    assert!(r.read_slot(0).is_some());
    a.undo(2_000);
    assert!(r.slots().iter().all(Option::is_none));
}

#[test]
fn undoing_a_save_touches_only_the_slot_it_wrote() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.write_slot(0, &[1u8; 8], b"png", "2026-08-09_00-00-00")
        .unwrap();
    r.set_active_slot(1).unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    a.undo(2_000);
    assert_eq!(r.read_slot(0).unwrap().1, [1u8; 8]);
    assert!(r.read_slot(1).is_none());
}

#[test]
fn undo_expires_after_thirty_seconds() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    assert!(a.undo_available(30_999));
    assert!(!a.undo_available(31_001));
}

#[test]
fn a_second_save_replaces_the_undo_rather_than_stacking() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    let mut a = app_playing_in(d.path(), "Emerald");
    r.write_slot(0, &[7u8; 64], b"png", "2026-08-09_00-00-00")
        .unwrap();
    a.apply_at(Action::SaveState, 1_000);
    a.apply_at(Action::SaveState, 2_000);
    a.undo(3_000);
    assert_eq!(
        r.read_slot(0).unwrap().1,
        [9u8; 1024],
        "only the most recent save should be undone"
    );
}

#[test]
fn undoing_a_load_returns_the_emulator_to_the_prior_state() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let core = CoreSnapshot::new();
    let mut a = app_playing_with(d.path(), "Emerald", core.boxed());
    core.run_frame();
    a.apply_at(Action::SaveState, 500);
    for _ in 0..120 {
        core.run_frame();
    }
    let before = core.bytes();
    a.apply_at(Action::LoadState, 1_000);
    assert_ne!(core.bytes(), before, "the load never moved the core");
    a.undo(2_000);
    assert_eq!(core.bytes(), before);
}

#[test]
fn undo_is_not_offered_when_nothing_is_undoable() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let a = app_playing_in(d.path(), "Emerald");
    assert!(!a.undo_available(0));
}

#[test]
fn the_label_names_what_will_be_undone() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    assert_eq!(a.undo_label(), Some("undo save"));
    a.apply_at(Action::LoadState, 2_000);
    assert_eq!(a.undo_label(), Some("undo load"));
}

#[test]
fn x_undoes_from_the_switcher_and_never_from_the_game() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    a.apply_at(Action::GbaDown(Btn::X), 1_100);
    assert!(r.read_slot(0).is_some(), "X undid from the game");

    a.apply_at(Action::Polaroids, 1_200);
    a.apply_at(Action::GbaDown(Btn::X), 1_300);
    assert!(r.read_slot(0).is_none(), "X did not undo the save");
}

#[test]
fn undoing_twice_does_not_put_the_save_back() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    a.undo(2_000);
    a.undo(3_000);
    assert!(r.read_slot(0).is_none());
    assert!(!a.undo_available(3_000));
}

#[test]
fn ejecting_the_cart_takes_its_undo_with_it() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    a.apply_at(Action::Eject, 1_100);
    assert!(!a.undo_available(1_200));
}

#[test]
fn the_plate_is_drawn_only_over_the_switcher_and_only_while_the_offer_stands() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    a.tick_ms(4_000);
    let mut playing = Vec::new();
    a.draw(&mut playing);
    assert!(
        a.undo_available(4_000),
        "the offer expired before it was tested"
    );
    assert!(playing.is_empty(), "the offer reached the game");

    a.apply_at(Action::Polaroids, 4_000);
    let mut open = Vec::new();
    a.draw(&mut open);
    a.tick_ms(40_000);
    let mut expired = Vec::new();
    a.draw(&mut expired);
    assert_eq!(
        open.len(),
        expired.len() + 1,
        "the plate outlived the offer"
    );
}

#[test]
fn undoing_a_save_made_from_the_picker_restores_the_slot_too() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.write_slot(2, &[7u8; 64], b"png", "2026-08-09_00-00-00")
        .unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::QuickMenu, 900);
    a.apply_at(Action::GbaDown(Btn::Down), 910);
    a.apply_at(Action::GbaDown(Btn::A), 920);
    a.apply_at(Action::GbaDown(Btn::Right), 930);
    a.apply_at(Action::GbaDown(Btn::Right), 940);
    a.apply_at(Action::GbaDown(Btn::A), 1_000);
    assert_eq!(r.read_slot(2).unwrap().1, [9u8; 1024]);
    a.undo(2_000);
    assert_eq!(r.read_slot(2).unwrap().1, [7u8; 64]);
}
