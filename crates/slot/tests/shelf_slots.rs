mod common;

use std::time::{Duration, Instant};

use common::{boot, clocked, tmp_root_with_carts};
use slot::app::{App, Phase};
use slot::persist::Snapshot;
use slot::session::Session;
use slot_input::{Action, Btn};
use slot_store::{Core, Platform, StateRing};
use slot_ui::SlotMode;
use tempfile::TempDir;

fn ring(d: &TempDir, stem: &str) -> StateRing {
    StateRing::new(d.path(), Platform::Gba, Core::Mgba, stem)
}

fn frame_state(frame: u64) -> Vec<u8> {
    frame.to_le_bytes().to_vec()
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn shelf(stems: &[&str]) -> (TempDir, App) {
    let d = tmp_root_with_carts(stems);
    let a = boot(d.path());
    assert!(matches!(a.phase(), Phase::Shelf));
    (d, a)
}

fn frame_of(s: &Session) -> u64 {
    let state = s.emu().expect("no core").snapshot().state().expect("state");
    u64::from_le_bytes(state.try_into().expect("a mock state is eight bytes"))
}

fn start_from_shelf(d: &TempDir, slot_presses: &[Btn]) -> Session {
    clocked(d.path());
    let mut s = Session::boot(d.path().to_path_buf());
    s.app_mut().apply(Action::GbaDown(Btn::Select));
    for b in slot_presses {
        s.app_mut().apply(Action::GbaDown(*b));
    }
    s.app_mut().apply(Action::GbaDown(Btn::A));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) || !s.frame_ready() {
        assert!(Instant::now() < deadline, "the cart never started");
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    s
}

#[test]
fn select_on_the_carousel_opens_the_picker_in_play_mode() {
    let (_d, mut a) = shelf(&["Emerald", "Fusion"]);
    assert!(!a.shelf_slots_open());
    press(&mut a, Btn::Select);
    assert!(a.shelf_slots_open());
    assert_eq!(a.slot_mode(), Some(SlotMode::Play));
    assert!(matches!(a.phase(), Phase::Shelf));
    assert_eq!(a.polaroid_entries().len(), 3);
}

#[test]
fn the_picker_shows_the_slots_of_the_selected_cart() {
    let (d, mut a) = shelf(&["Emerald", "Fusion"]);
    ring(&d, "Fusion")
        .write_slot(1, &frame_state(9), b"png", "2026-08-09_00-00-00")
        .unwrap();
    press(&mut a, Btn::Select);
    assert!(a.polaroid_entries().iter().all(Option::is_none));
    press(&mut a, Btn::Select);
    assert!(!a.shelf_slots_open());
    a.apply(Action::ShelfRight);
    press(&mut a, Btn::Select);
    let shown = a.polaroid_entries();
    assert!(shown[0].is_none() && shown[1].is_some() && shown[2].is_none());
}

#[test]
fn b_select_and_menu_all_close_the_shelf_picker() {
    for close in [
        Action::GbaDown(Btn::B),
        Action::GbaDown(Btn::Select),
        Action::QuickMenu,
    ] {
        let (_d, mut a) = shelf(&["Emerald", "Fusion"]);
        press(&mut a, Btn::Select);
        a.apply(close);
        assert!(!a.shelf_slots_open(), "{close:?}");
        assert!(matches!(a.phase(), Phase::Shelf), "{close:?}");
        assert_eq!(a.quick_menu(), None, "{close:?}");
    }
}

#[test]
fn the_shelf_picker_moves_between_slots_and_stops_at_the_ends() {
    let (_d, mut a) = shelf(&["Emerald", "Fusion"]);
    press(&mut a, Btn::Select);
    for _ in 0..5 {
        press(&mut a, Btn::Right);
    }
    assert_eq!(a.polaroid_title("2026-08-09_00-00-00"), "Slot 3 · Empty");
    for _ in 0..5 {
        press(&mut a, Btn::Left);
    }
    assert_eq!(a.polaroid_title("2026-08-09_00-00-00"), "Slot 1 · Empty");
}

#[test]
fn the_carousel_does_not_move_under_the_open_picker() {
    let (_d, mut a) = shelf(&["Emerald", "Fusion"]);
    press(&mut a, Btn::Select);
    press(&mut a, Btn::Right);
    a.apply(Action::ShelfRight);
    a.apply(Action::GbaDown(Btn::Start));
    assert_eq!(a.selected_stem(), Some("Emerald"));
    assert!(a.shelf_slots_open());
}

#[test]
fn the_slot_highlighted_in_the_shelf_picker_is_remembered_as_the_active_one() {
    let (d, mut a) = shelf(&["Emerald", "Fusion"]);
    press(&mut a, Btn::Select);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::B);
    assert_eq!(ring(&d, "Emerald").active_slot(), 2);
}

#[test]
fn a_on_a_filled_slot_inserts_the_cart_and_hands_over_that_slots_state() {
    let (d, mut a) = shelf(&["Emerald", "Fusion"]);
    ring(&d, "Emerald")
        .write_slot(0, &frame_state(1), b"png", "2026-08-09_00-00-00")
        .unwrap();
    press(&mut a, Btn::Select);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
    assert!(!a.shelf_slots_open());
    let handed = a.take_start_state().expect("no state was handed over");
    assert_eq!(std::fs::read(handed).unwrap(), frame_state(1));
    assert_eq!(a.take_start_state(), None, "the state is handed over once");
}

#[test]
fn a_on_an_empty_slot_inserts_normally_with_no_state() {
    let (_d, mut a) = shelf(&["Emerald", "Fusion"]);
    press(&mut a, Btn::Select);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
    assert_eq!(a.take_start_state(), None);
}

#[test]
fn the_core_starts_from_the_chosen_slot_and_not_from_the_resume() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let r = ring(&d, "Emerald");
    r.write_resume(&frame_state(7_000_000)).unwrap();
    r.write_slot(1, &frame_state(5_000_000), b"png", "2026-08-09_00-00-00")
        .unwrap();
    let s = start_from_shelf(&d, &[Btn::Right]);
    let frame = frame_of(&s);
    assert!(
        (5_000_000..6_000_000).contains(&frame),
        "the core started at frame {frame}"
    );
}

#[test]
fn the_core_starts_from_its_resume_when_the_slot_chosen_is_empty() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let r = ring(&d, "Emerald");
    r.write_resume(&frame_state(7_000_000)).unwrap();
    r.write_slot(1, &frame_state(5_000_000), b"png", "2026-08-09_00-00-00")
        .unwrap();
    let s = start_from_shelf(&d, &[]);
    let frame = frame_of(&s);
    assert!(
        (7_000_000..8_000_000).contains(&frame),
        "the core started at frame {frame}"
    );
}

#[test]
fn a_slot_start_does_not_leak_into_the_next_insert() {
    let (d, mut a) = shelf(&["Emerald", "Fusion"]);
    ring(&d, "Emerald")
        .write_slot(0, &frame_state(1), b"png", "2026-08-09_00-00-00")
        .unwrap();
    press(&mut a, Btn::Select);
    press(&mut a, Btn::A);
    assert!(a.take_start_state().is_some());
    a.apply(Action::Eject);
    for _ in 0..300 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::Insert);
    assert_eq!(a.take_start_state(), None);
}
