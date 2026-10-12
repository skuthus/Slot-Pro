mod common;

use common::{
    app_booting_at, app_playing_with, pad_of, session_playing, tmp_root_with_carts, StubSnapshot,
    CLOCK_IS_SET,
};
use slot::app::{App, Phase};
use slot::persist;
use slot::persist::Snapshot;
use slot_input::{Action, Btn, RawEvent};
use slot_store::{read_slot_state, write_slot_state, Core, Platform, SlotState, StateRing};
use slot_ui::{QuickRow, QuickValue};
use tempfile::TempDir;

fn analog_state(stem: &str) -> SlotState {
    SlotState {
        cart: Some(stem.to_string()),
        clock_set: true,
        utc_offset_min: 0,
        analog: true,
        ..Default::default()
    }
}

fn snapshot_with_sav() -> Box<dyn Snapshot> {
    Box::new(StubSnapshot {
        state: vec![9u8; 1024],
        sav: Some(b"savdata".to_vec()),
        thumb: Some(b"png".to_vec()),
        loaded: Default::default(),
    })
}

fn analog_app(d: &TempDir) -> App {
    write_slot_state(d.path(), &analog_state("Emerald")).unwrap();
    let mut a = App::boot(d.path());
    a.set_snapshot(snapshot_with_sav());
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "{:?}",
        a.phase()
    );
    a
}

fn ring(d: &TempDir) -> StateRing {
    StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald")
}

fn no_slots(d: &TempDir) -> bool {
    ring(d).slots().iter().all(Option::is_none) && ring(d).list().unwrap().is_empty()
}

fn sav(d: &TempDir) -> Option<Vec<u8>> {
    std::fs::read(d.path().join("Saves/GBA/Emerald.sav")).ok()
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

#[test]
fn the_analog_row_leads_the_gameplay_page() {
    assert_eq!(QuickRow::GAME[0], QuickRow::Analog);
    assert_eq!(QuickRow::Analog.label(), "Analog Mode");
    assert_eq!(QuickRow::Analog.parent(), Some(QuickRow::Game));
    assert_eq!(QuickRow::Game.child(), Some(QuickRow::Analog));
    assert_eq!(QuickRow::FastForward.parent(), Some(QuickRow::Game));
}

#[test]
fn the_analog_row_toggles_on_either_arrow_and_persists() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    assert!(!a.analog());
    assert_eq!(a.quick_value(QuickRow::Analog), Some(QuickValue::Off));
    a.apply(Action::QuickMenu);
    for _ in 0..QuickRow::Game.position() {
        press(&mut a, Btn::Down);
    }
    assert_eq!(a.quick_menu(), Some(QuickRow::Game));
    press(&mut a, Btn::A);
    assert_eq!(a.quick_menu(), Some(QuickRow::Analog));
    press(&mut a, Btn::Right);
    assert!(a.analog());
    assert_eq!(a.quick_value(QuickRow::Analog), Some(QuickValue::On));
    assert!(read_slot_state(d.path()).analog, "never reached the card");
    press(&mut a, Btn::Left);
    assert!(!a.analog());
    assert!(!read_slot_state(d.path()).analog);
    press(&mut a, Btn::Left);
    assert!(a.analog(), "left did not flip it too");
    assert!(read_slot_state(d.path()).analog);
}

#[test]
fn turbo_rewind_and_fast_forward_are_withheld_in_analog() {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_slot_state(
        d.path(),
        &SlotState {
            turbo: true,
            rewind: true,
            ..analog_state("Emerald")
        },
    )
    .unwrap();
    let a = App::boot(d.path());
    assert!(!a.turbo());
    assert!(!a.records_rewind());
    assert!(!a.may_rewind());
    assert!(!a.may_fast_forward());

    let d = tmp_root_with_carts(&["Emerald"]);
    write_slot_state(
        d.path(),
        &SlotState {
            turbo: true,
            rewind: true,
            cart: Some("Emerald".into()),
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let a = App::boot(d.path());
    assert!(a.turbo() && a.records_rewind() && a.may_rewind() && a.may_fast_forward());
}

#[test]
fn the_four_in_game_actions_do_nothing_in_analog() {
    for action in [
        Action::QuickMenu,
        Action::Polaroids,
        Action::SaveState,
        Action::LoadState,
    ] {
        let d = tmp_root_with_carts(&["Emerald"]);
        let mut a = analog_app(&d);
        a.apply(action);
        a.settle_saves();
        assert!(
            matches!(a.phase(), Phase::Playing { .. }),
            "{action:?}: {:?}",
            a.phase()
        );
        assert_eq!(a.paused(), None, "{action:?} opened the pause menu");
        assert!(!a.slots_open(), "{action:?} opened a picker");
        assert!(a.toast().is_none(), "{action:?} toasted");
        assert_eq!(a.screen_shake(), 0.0, "{action:?} shook");
        assert!(
            ring(&d).list().unwrap().is_empty(),
            "{action:?} wrote a slot"
        );
        assert!(ring(&d).read_resume().unwrap().is_none());
    }
}

#[test]
fn the_same_actions_work_with_analog_off() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_with(d.path(), "Emerald", StubSnapshot::boxed());
    a.apply(Action::SaveState);
    a.settle_saves();
    assert!(ring(&d).slots()[0].is_some(), "control: save wrote nothing");
    a.apply(Action::QuickMenu);
    assert!(a.paused().is_some(), "control: no pause menu");
}

#[test]
fn rewind_and_fast_forward_presses_are_ignored_without_a_refusal_in_analog() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = analog_app(&d);
    a.apply(Action::RewindStart);
    a.apply(Action::FfStart);
    assert_eq!(a.screen_shake(), 0.0);
    assert!(!a.refusal_active(a.now()), "a refusal was started");
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}

#[test]
fn hold_menu_eject_still_works_in_analog_and_keeps_the_sav_only() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_slot_state(
        d.path(),
        &SlotState {
            eject_save: true,
            ..analog_state("Emerald")
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_snapshot(snapshot_with_sav());
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    a.apply(Action::Eject);
    for _ in 0..240 {
        a.update(1.0 / 60.0);
    }
    a.settle_saves();
    assert!(
        matches!(a.phase(), Phase::Shelf),
        "eject did not leave the game"
    );
    assert!(
        ring(&d).read_resume().unwrap().is_none(),
        "eject wrote a resume with eject_save on"
    );
    assert_eq!(
        sav(&d).as_deref(),
        Some(&b"savdata"[..]),
        "the .sav was lost"
    );
}

#[test]
fn flush_resume_and_power_paths_write_the_sav_but_no_resume_in_analog() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = analog_app(&d);
    a.flush_resume();
    assert!(ring(&d).read_resume().unwrap().is_none(), "flush_resume");
    assert_eq!(
        sav(&d).as_deref(),
        Some(&b"savdata"[..]),
        "flush_resume sav"
    );
    std::fs::remove_file(d.path().join("Saves/GBA/Emerald.sav")).unwrap();

    a.apply(Action::PowerTap);
    assert!(ring(&d).read_resume().unwrap().is_none(), "power press");
    assert_eq!(sav(&d).as_deref(), Some(&b"savdata"[..]), "power press sav");
    std::fs::remove_file(d.path().join("Saves/GBA/Emerald.sav")).unwrap();

    a.apply(Action::GbaDown(Btn::A));
    assert!(matches!(a.phase(), Phase::Doze { .. }));
    assert!(ring(&d).read_resume().unwrap().is_none(), "doze");
    assert_eq!(sav(&d).as_deref(), Some(&b"savdata"[..]), "doze sav");
}

#[test]
fn autosave_writes_the_sav_but_no_resume_in_analog() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = analog_app(&d);
    a.tick_ms(60_000);
    a.settle_saves();
    assert!(ring(&d).read_resume().unwrap().is_none());
    assert_eq!(sav(&d).as_deref(), Some(&b"savdata"[..]));
}

#[test]
fn analog_off_still_writes_a_resume_on_autosave() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_with(d.path(), "Emerald", snapshot_with_sav());
    a.tick_ms(60_000);
    a.settle_saves();
    assert!(ring(&d).read_resume().unwrap().is_some());
}

#[test]
fn analog_always_starts_clean() {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_slot_state(d.path(), &analog_state("Emerald")).unwrap();
    assert!(App::boot(d.path()).starting_clean());
    let d = tmp_root_with_carts(&["Emerald"]);
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Emerald".into()),
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!App::boot(d.path()).starting_clean());
}

#[test]
fn the_carousel_select_does_not_open_the_slot_picker_in_analog() {
    for (analog, want) in [(true, false), (false, true)] {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        write_slot_state(
            d.path(),
            &SlotState {
                clock_set: true,
                analog,
                ..Default::default()
            },
        )
        .unwrap();
        let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
        assert!(matches!(a.phase(), Phase::Shelf));
        a.apply(Action::GbaDown(Btn::Select));
        assert_eq!(a.shelf_slots_open(), want, "analog={analog}");
    }
}

fn resume_counter(session: &mut slot::session::Session) -> u64 {
    let state = session
        .emu()
        .expect("no core")
        .request_state()
        .recv()
        .expect("state");
    u64::from_le_bytes(state.try_into().expect("mock state is 8 bytes"))
}

fn seed_resume(d: &TempDir) {
    persist::flush(
        d.path(),
        Platform::Gba,
        Core::Mgba,
        "Emerald",
        Some(&500_000u64.to_le_bytes()),
        None,
    )
    .unwrap();
}

#[test]
fn a_game_boots_fresh_in_analog_and_resumes_otherwise() {
    for (analog, resumes) in [(true, false), (false, true)] {
        let d = tmp_root_with_carts(&["Emerald"]);
        seed_resume(&d);
        write_slot_state(
            d.path(),
            &SlotState {
                clock_set: true,
                analog,
                ..Default::default()
            },
        )
        .unwrap();
        let mut s = session_playing(d.path());
        let n = resume_counter(&mut s);
        assert_eq!(n >= 500_000, resumes, "analog={analog}, counter {n}");
        assert!(
            ring(&d).read_resume().unwrap().is_some(),
            "the old resume was deleted"
        );
    }
}

#[test]
fn select_plus_a_shoulder_reaches_the_game_in_analog_and_chords_otherwise() {
    for analog in [true, false] {
        let d = tmp_root_with_carts(&["Emerald"]);
        write_slot_state(
            d.path(),
            &SlotState {
                clock_set: true,
                analog,
                ..Default::default()
            },
        )
        .unwrap();
        let mut s = session_playing(d.path());
        s.feed([RawEvent::Down(Btn::Select)], 1_000);
        s.feed([RawEvent::Down(Btn::R1)], 1_050);
        let pad = pad_of(&s);
        assert_eq!(pad != 0, analog, "analog={analog}, pad {pad:#x}");
        s.update(1.0 / 60.0);
        if !analog {
            assert!(ring(&d).slots()[0].is_some(), "the chord saved nothing");
        } else {
            assert!(no_slots(&d));
            assert!(s.app().toast().is_none());
        }
    }
}
