mod common;

use common::{app_playing_in, app_playing_with, tmp_root_with_carts, StubSnapshot};
use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{Cart, Core, Platform, StateRing};
use slot_ui::{SlotMode, Toast};

fn ring(d: &std::path::Path) -> StateRing {
    StateRing::new(d, Platform::Gba, Core::Mgba, "Emerald")
}

fn app_without_a_card(stem: &str) -> App {
    let mut a = App::new(vec![Cart {
        platform: Platform::Gba,
        stem: stem.to_string(),
        rom: format!("Games/GBA/{stem}.gba").into(),
        label: None,
        code: String::new(),
        shell: None,
        title: stem.to_uppercase(),
    }]);
    a.apply(Action::Insert);
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a
}

#[test]
fn double_tap_menu_with_no_card_does_not_open_the_picker() {
    let mut a = app_without_a_card("Emerald");
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn double_tap_menu_opens_the_picker_in_load_mode_even_with_nothing_saved() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    assert_eq!(a.slot_mode(), Some(SlotMode::Load));
    assert_eq!(a.polaroid_entries().len(), 3);
    assert!(a.polaroid_entries().iter().all(Option::is_none));
}

#[test]
fn saving_writes_the_state_and_its_picture_into_slot_one() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    let slots = ring(d.path()).slots();
    let entry = slots[0].as_ref().expect("slot 1 is empty");
    assert!(slots[1].is_none() && slots[2].is_none());
    assert_eq!(std::fs::read(&entry.state).expect("state"), [9u8; 1024]);
    assert!(
        !std::fs::read(&entry.thumb).expect("thumb").is_empty(),
        "the slot has no picture"
    );
}

#[test]
fn saving_twice_keeps_one_state_in_the_slot() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    a.apply(Action::SaveState);
    let files = std::fs::read_dir(d.path().join("States/GBA/mgba/Emerald"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("slot1-"))
        .count();
    assert_eq!(files, 2, "one state and one picture");
}

#[test]
fn a_saved_slot_opens_in_the_picker_and_a_loads_it_back() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (snapshot, loaded) = StubSnapshot::pair();
    let mut a = app_playing_with(d.path(), "Emerald", snapshot);
    a.apply(Action::SaveState);
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
    assert!(a.polaroid_entries()[0].is_some());
    a.apply(Action::GbaDown(Btn::A));
    assert_eq!(
        loaded.lock().expect("loaded").as_deref(),
        Some(&[9u8; 1024][..])
    );
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "loading must hand the game back"
    );
}

#[test]
fn b_dismisses_the_picker_without_loading() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (snapshot, loaded) = StubSnapshot::pair();
    let mut a = app_playing_with(d.path(), "Emerald", snapshot);
    a.apply(Action::SaveState);
    a.apply(Action::Polaroids);
    a.apply(Action::GbaDown(Btn::B));
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert!(loaded.lock().expect("loaded").is_none());
}

#[test]
fn menu_dismisses_the_picker_it_opened() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    a.apply(Action::Polaroids);
    a.apply(Action::Polaroids);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert!(!a.slots_open());
}

#[test]
fn flicking_selects_which_slot_a_loads() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = ring(d.path());
    r.write_slot(0, &[1u8; 16], b"png", "2026-08-09_00-00-01")
        .unwrap();
    r.write_slot(1, &[2u8; 16], b"png", "2026-08-09_00-00-02")
        .unwrap();
    let (snapshot, loaded) = StubSnapshot::pair();
    let mut a = app_playing_with(d.path(), "Emerald", snapshot);
    a.apply(Action::Polaroids);
    a.apply(Action::GbaDown(Btn::Right));
    a.apply(Action::GbaDown(Btn::A));
    assert_eq!(
        loaded.lock().expect("loaded").as_deref(),
        Some(&[2u8; 16][..])
    );
}

#[test]
fn the_picker_stops_at_the_last_slot() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Polaroids);
    for _ in 0..5 {
        a.apply(Action::GbaDown(Btn::Right));
    }
    assert_eq!(a.polaroid_title("2026-08-09_00-00-00"), "Slot 3 · Empty");
    for _ in 0..5 {
        a.apply(Action::GbaDown(Btn::Left));
    }
    assert_eq!(a.polaroid_title("2026-08-09_00-00-00"), "Slot 1 · Empty");
}

#[test]
fn loading_an_empty_slot_from_the_picker_refuses_and_stays_open() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (snapshot, loaded) = StubSnapshot::pair();
    let mut a = app_playing_with(d.path(), "Emerald", snapshot);
    a.apply(Action::Polaroids);
    a.apply(Action::GbaDown(Btn::A));
    assert!(loaded.lock().expect("loaded").is_none());
    assert!(
        matches!(a.phase(), Phase::Polaroids { .. }),
        "the picker closed on an empty slot"
    );
    assert!(a.refusal_active(a.now()), "nothing shook");
}

#[test]
fn load_state_takes_the_active_slot_without_the_picker() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let r = ring(d.path());
    r.write_slot(0, &[1u8; 16], b"png", "2026-08-09_00-00-01")
        .unwrap();
    r.write_slot(2, &[3u8; 16], b"png", "2026-08-09_00-00-02")
        .unwrap();
    r.set_active_slot(2).unwrap();
    let (snapshot, loaded) = StubSnapshot::pair();
    let mut a = app_playing_with(d.path(), "Emerald", snapshot);
    a.apply(Action::LoadState);
    assert_eq!(
        loaded.lock().expect("loaded").as_deref(),
        Some(&[3u8; 16][..])
    );
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    assert_eq!(a.toast(), Some(Toast::SlotLoaded(2)));
}

#[test]
fn load_state_on_an_empty_active_slot_refuses_and_loads_nothing() {
    let d = tmp_root_with_carts(&["Emerald"]);
    ring(d.path())
        .write_slot(0, &[1u8; 16], b"png", "2026-08-09_00-00-01")
        .unwrap();
    ring(d.path()).set_active_slot(1).unwrap();
    let (snapshot, loaded) = StubSnapshot::pair();
    let mut a = app_playing_with(d.path(), "Emerald", snapshot);
    a.apply(Action::LoadState);
    assert!(loaded.lock().expect("loaded").is_none());
    assert!(a.refusal_active(a.now()));
    assert_eq!(a.toast(), None);
}

#[test]
fn save_state_goes_to_the_active_slot_and_makes_it_stay_active() {
    let d = tmp_root_with_carts(&["Emerald"]);
    ring(d.path()).set_active_slot(1).unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    let slots = ring(d.path()).slots();
    assert!(slots[0].is_none() && slots[1].is_some() && slots[2].is_none());
    assert_eq!(a.toast(), Some(Toast::SlotSaved(1)));
    assert_eq!(ring(d.path()).active_slot(), 1);
}

#[test]
fn the_picker_opens_on_the_active_slot() {
    let d = tmp_root_with_carts(&["Emerald"]);
    ring(d.path()).set_active_slot(2).unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Polaroids);
    assert_eq!(a.polaroid_title("2026-08-09_00-00-00"), "Slot 3 · Empty");
}

#[test]
fn leaving_the_picker_remembers_the_slot_it_was_on() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Polaroids);
    a.apply(Action::GbaDown(Btn::Right));
    a.apply(Action::GbaDown(Btn::B));
    assert_eq!(ring(d.path()).active_slot(), 1);
    a.apply(Action::SaveState);
    assert!(ring(d.path()).slots()[1].is_some());
}

#[test]
fn the_picker_swallows_the_shelf_and_eject_bindings() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    a.apply(Action::Polaroids);
    a.apply(Action::Eject);
    a.apply(Action::Insert);
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
}

#[test]
fn there_is_no_delete_in_the_picker() {
    let d = tmp_root_with_carts(&["Emerald"]);
    ring(d.path())
        .write_slot(0, &[1u8; 16], b"png", "2026-08-09_00-00-01")
        .unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Polaroids);
    a.apply(Action::GbaDown(Btn::Y));
    assert!(ring(d.path()).slots()[0].is_some());
    assert!(matches!(a.phase(), Phase::Polaroids { .. }));
}
