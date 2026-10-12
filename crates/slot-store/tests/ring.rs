use slot_store::{Core, Platform, StateRing, RING_MAX, SLOT_COUNT};
use tempfile::tempdir;

#[test]
fn ring_evicts_the_oldest_beyond_ten() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    for i in 0..14 {
        r.push(&[i as u8; 64], b"png", &format!("2026-08-09_00-00-{i:02}"))
            .unwrap();
    }
    let l = r.list().unwrap();
    assert_eq!(l.len(), RING_MAX);
    assert_eq!(l[0].stamp, "2026-08-09_00-00-13");
    assert_eq!(l[9].stamp, "2026-08-09_00-00-04");
    let orphans = std::fs::read_dir(d.path().join("States/GBA/mgba/Emerald"))
        .unwrap()
        .count();
    assert_eq!(orphans, RING_MAX * 2, "evicted thumbnails were not deleted");
}

#[test]
fn resume_is_not_part_of_the_ring() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    for i in 0..12 {
        r.write_resume(&[i; 8]).unwrap();
    }
    assert!(
        r.list().unwrap().is_empty(),
        "resume writes leaked into the ring"
    );
    assert_eq!(r.read_resume().unwrap().unwrap(), vec![11u8; 8]);
}

#[test]
fn a_cart_with_no_saves_lists_empty_rather_than_erroring() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Never Played");
    assert!(r.list().unwrap().is_empty());
    assert!(r.read_resume().unwrap().is_none());
}

#[test]
fn eviction_is_by_stamp_not_by_write_order() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    for i in [9, 3, 7, 1, 5, 11, 0, 8, 2, 10, 6, 4] {
        r.push(&[i as u8; 8], b"png", &format!("2026-08-09_00-00-{i:02}"))
            .unwrap();
    }
    let l = r.list().unwrap();
    assert_eq!(l.len(), RING_MAX);
    assert_eq!(l[0].stamp, "2026-08-09_00-00-11");
    assert_eq!(l[RING_MAX - 1].stamp, "2026-08-09_00-00-02");
}

#[test]
fn files_that_are_not_stamped_states_are_neither_listed_nor_evicted() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.push(&[1u8; 8], b"png", "2026-08-09_00-00-01").unwrap();
    let dir = d.path().join("States/GBA/mgba/Emerald");
    let strays = [
        dir.join("2026-08-09_09-09-09.png"),
        dir.join("notes.txt"),
        dir.join("notes.state"),
    ];
    for s in &strays {
        std::fs::write(s, b"stray").unwrap();
    }

    let l = r.list().unwrap();
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].stamp, "2026-08-09_00-00-01");

    for i in 0..RING_MAX + 4 {
        r.push(&[0u8; 8], b"png", &format!("2026-08-09_01-00-{i:02}"))
            .unwrap();
    }
    for s in &strays {
        assert!(s.exists(), "eviction deleted {s:?}");
    }
}

#[test]
fn read_pairs_the_state_with_its_thumbnail() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.push(&[1u8; 8], b"first", "2026-08-09_00-00-01").unwrap();
    r.push(&[2u8; 8], b"second", "2026-08-09_00-00-02").unwrap();
    let (state, thumb) = r.read("2026-08-09_00-00-01").unwrap();
    assert_eq!(state, [1u8; 8]);
    assert_eq!(thumb, b"first");
}

#[test]
fn read_of_an_entry_with_no_thumbnail_is_the_state_and_nothing_else() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.push(&[1u8; 8], b"png", "2026-08-09_00-00-01").unwrap();
    std::fs::remove_file(
        d.path()
            .join("States/GBA/mgba/Emerald/2026-08-09_00-00-01.png"),
    )
    .unwrap();
    let (state, thumb) = r.read("2026-08-09_00-00-01").unwrap();
    assert_eq!(state, [1u8; 8]);
    assert!(thumb.is_empty());
}

#[test]
fn remove_takes_the_thumbnail_with_the_state() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.push(&[1u8; 8], b"png", "2026-08-09_00-00-01").unwrap();
    r.remove("2026-08-09_00-00-01").unwrap();
    assert!(r.list().unwrap().is_empty());
    assert_eq!(
        std::fs::read_dir(d.path().join("States/GBA/mgba/Emerald"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn remove_refuses_anything_that_is_not_a_stamp() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.write_resume(&[7u8; 8]).unwrap();
    assert!(r.remove("resume").is_err());
    assert!(r.read("resume").is_err());
    assert_eq!(r.read_resume().unwrap().unwrap(), [7u8; 8]);
}

#[test]
fn a_sidecar_is_not_listed_as_a_state() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.push(b"state", b"png", "2026-08-09_00-00-01").unwrap();
    let dir = d.path().join("States/GBA/mgba/Emerald");
    std::fs::write(dir.join("._2026-08-09_00-00-01.state"), b"sidecar").unwrap();
    let l = r.list().unwrap();
    assert_eq!(l.len(), 1, "a sidecar was listed as a save state");
    assert_eq!(l[0].stamp, "2026-08-09_00-00-01");
}

#[test]
fn each_core_keeps_its_own_states() {
    let d = tempdir().unwrap();
    let mgba = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    let gpsp = StateRing::new(d.path(), Platform::Gba, Core::Gpsp, "Emerald");

    mgba.push(&[1u8; 64], b"png", "2026-08-21_00-00-01")
        .unwrap();

    assert_eq!(mgba.list().unwrap().len(), 1);
    assert_eq!(
        gpsp.list().unwrap().len(),
        0,
        "a save state is not portable between cores and must not be offered as if it were"
    );
    assert!(d.path().join("States/GBA/mgba/Emerald").is_dir());
    assert!(!d.path().join("States/Emerald").exists());
}

#[test]
fn resume_is_core_private_too() {
    let d = tempdir().unwrap();
    StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald")
        .write_resume(&[9u8; 16])
        .unwrap();
    assert!(
        StateRing::new(d.path(), Platform::Gba, Core::Gpsp, "Emerald")
            .read_resume()
            .unwrap()
            .is_none()
    );
}

#[test]
fn each_platform_keeps_its_own_states() {
    let d = tempdir().unwrap();
    let gba = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Tetris");
    let gb = StateRing::new(d.path(), Platform::Gb, Core::Mgba, "Tetris");

    gba.push(&[1u8; 64], b"png", "2026-08-21_00-00-01").unwrap();

    assert_eq!(gba.list().unwrap().len(), 1);
    assert_eq!(
        gb.list().unwrap().len(),
        0,
        "a save state is not portable between platforms and must not be offered as if it were"
    );
    assert!(d.path().join("States/GBA/mgba/Tetris").is_dir());
    assert!(!d.path().join("States/GB/mgba/Tetris").exists());
}

#[test]
fn a_retired_resume_stops_being_read_back_but_is_still_there() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.write_resume(&[7u8; 64]).unwrap();

    let to = r
        .retire_resume("2026-09-16_23-30-00")
        .unwrap()
        .expect("nothing was moved");

    assert!(r.read_resume().unwrap().is_none(), "still offered");
    assert_eq!(std::fs::read(&to).unwrap(), [7u8; 64], "the bytes changed");
    assert_eq!(
        to.file_name().unwrap().to_str().unwrap(),
        "resume-refused-2026-09-16_23-30-00.state"
    );
    assert_eq!(
        to.parent().unwrap(),
        d.path().join("States/GBA/mgba/Emerald"),
        "the retired state left the cart's own directory"
    );
}

#[test]
fn retiring_a_cart_with_no_resume_does_nothing() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    assert!(r.retire_resume("2026-09-16_23-30-00").unwrap().is_none());
}

#[test]
fn a_retired_resume_is_not_a_ring_entry() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    r.push(b"state", b"png", "2026-08-09_00-00-01").unwrap();
    r.write_resume(&[7u8; 64]).unwrap();
    r.retire_resume("2026-09-16_23-30-00").unwrap();

    let l = r.list().unwrap();
    assert_eq!(l.len(), 1, "the retired state was listed as a save state");
    assert_eq!(l[0].stamp, "2026-08-09_00-00-01");
}

#[test]
fn a_second_retirement_in_the_same_second_does_not_overwrite_the_first() {
    let d = tempdir().unwrap();
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");

    r.write_resume(&[1u8; 8]).unwrap();
    let first = r.retire_resume("2026-09-16_23-30-00").unwrap().unwrap();
    r.write_resume(&[2u8; 8]).unwrap();
    let second = r.retire_resume("2026-09-16_23-30-00").unwrap().unwrap();

    assert_ne!(first, second, "the second retirement took the same name");
    assert_eq!(std::fs::read(&first).unwrap(), [1u8; 8], "the first went");
    assert_eq!(std::fs::read(&second).unwrap(), [2u8; 8]);
}

fn slot_ring(d: &std::path::Path) -> StateRing {
    StateRing::new(d, Platform::Gba, Core::Mgba, "Emerald")
}

fn slot_files(d: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(d.join("States/GBA/mgba/Emerald"))
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_cart_with_no_slots_has_three_empty_ones() {
    let d = tempdir().unwrap();
    let slots = slot_ring(d.path()).slots();
    assert_eq!(slots.len(), SLOT_COUNT);
    assert!(slots.iter().all(Option::is_none));
}

#[test]
fn a_slot_reads_back_what_was_written_into_it() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(1, b"state", b"png", "2026-08-09_00-00-01")
        .unwrap();
    let (stamp, state, thumb) = r.read_slot(1).unwrap();
    assert_eq!(stamp, "2026-08-09_00-00-01");
    assert_eq!(state, b"state");
    assert_eq!(thumb, b"png");
    let slots = r.slots();
    assert!(slots[0].is_none() && slots[2].is_none());
    assert_eq!(slots[1].as_ref().unwrap().stamp, "2026-08-09_00-00-01");
    assert!(r.read_slot(0).is_none());
}

#[test]
fn slot_files_are_named_with_a_one_based_number() {
    let d = tempdir().unwrap();
    slot_ring(d.path())
        .write_slot(0, b"s", b"p", "2026-08-09_00-00-01")
        .unwrap();
    assert_eq!(
        slot_files(d.path()),
        [
            "slot1-2026-08-09_00-00-01.png",
            "slot1-2026-08-09_00-00-01.state"
        ]
    );
}

#[test]
fn overwriting_a_slot_keeps_one_file_pair_and_the_newer_bytes() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(2, b"old", b"old-png", "2026-08-09_00-00-01")
        .unwrap();
    r.write_slot(2, b"new", b"new-png", "2026-08-09_00-00-02")
        .unwrap();
    assert_eq!(slot_files(d.path()).len(), 2, "{:?}", slot_files(d.path()));
    let (stamp, state, thumb) = r.read_slot(2).unwrap();
    assert_eq!(stamp, "2026-08-09_00-00-02");
    assert_eq!(
        (state.as_slice(), thumb.as_slice()),
        (&b"new"[..], &b"new-png"[..])
    );
}

#[test]
fn rewriting_a_slot_with_the_same_stamp_replaces_it_in_place() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(0, b"one", b"p", "2026-08-09_00-00-01")
        .unwrap();
    r.write_slot(0, b"two", b"p", "2026-08-09_00-00-01")
        .unwrap();
    assert_eq!(slot_files(d.path()).len(), 2);
    assert_eq!(r.read_slot(0).unwrap().1, b"two");
}

#[test]
fn writing_one_slot_leaves_the_others_alone() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(0, b"a", b"p", "2026-08-09_00-00-01").unwrap();
    r.write_slot(1, b"b", b"p", "2026-08-09_00-00-02").unwrap();
    r.write_slot(0, b"c", b"p", "2026-08-09_00-00-03").unwrap();
    assert_eq!(r.read_slot(0).unwrap().1, b"c");
    assert_eq!(r.read_slot(1).unwrap().1, b"b");
    assert_eq!(slot_files(d.path()).len(), 4);
}

#[test]
fn clearing_a_slot_takes_the_state_and_the_thumbnail_and_nothing_else() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(0, b"a", b"p", "2026-08-09_00-00-01").unwrap();
    r.write_slot(1, b"b", b"p", "2026-08-09_00-00-02").unwrap();
    r.clear_slot(0).unwrap();
    assert!(r.read_slot(0).is_none());
    assert!(r.read_slot(1).is_some());
    assert_eq!(slot_files(d.path()).len(), 2);
}

#[test]
fn clearing_an_empty_slot_of_a_cart_never_saved_is_not_an_error() {
    let d = tempdir().unwrap();
    slot_ring(d.path()).clear_slot(1).unwrap();
}

#[test]
fn a_slot_without_a_thumbnail_still_reads_back_its_state() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(0, b"a", b"p", "2026-08-09_00-00-01").unwrap();
    std::fs::remove_file(
        d.path()
            .join("States/GBA/mgba/Emerald/slot1-2026-08-09_00-00-01.png"),
    )
    .unwrap();
    let (_, state, thumb) = r.read_slot(0).unwrap();
    assert_eq!(state, b"a");
    assert!(thumb.is_empty());
}

#[test]
fn slots_are_not_part_of_the_old_ring_and_ring_entries_are_not_slots() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(0, b"a", b"p", "2026-08-09_00-00-01").unwrap();
    assert!(r.list().unwrap().is_empty(), "a slot leaked into the ring");
    r.push(b"b", b"p", "2026-08-09_00-00-02").unwrap();
    assert!(r.slots()[1].is_none() && r.slots()[2].is_none());
}

#[test]
fn files_that_are_not_slot_states_are_ignored() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.write_slot(0, b"a", b"p", "2026-08-09_00-00-01").unwrap();
    let dir = d.path().join("States/GBA/mgba/Emerald");
    for name in [
        "slot4-2026-08-09_00-00-01.state",
        "slot0-2026-08-09_00-00-01.state",
        "slotx-2026-08-09_00-00-01.state",
        "slot2-notastamp.state",
    ] {
        std::fs::write(dir.join(name), b"x").unwrap();
    }
    let slots = r.slots();
    assert!(slots[1].is_none() && slots[2].is_none());
    r.clear_slot(0).unwrap();
    assert!(
        dir.join("slot4-2026-08-09_00-00-01.state").exists(),
        "an out of range file was swept up"
    );
}

#[test]
fn the_active_slot_is_the_first_until_one_is_chosen() {
    let d = tempdir().unwrap();
    assert_eq!(slot_ring(d.path()).active_slot(), 0);
}

#[test]
fn the_active_slot_round_trips_as_a_one_based_number() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.set_active_slot(2).unwrap();
    assert_eq!(r.active_slot(), 2);
    assert_eq!(
        std::fs::read_to_string(d.path().join("States/GBA/mgba/Emerald/active-slot.txt"))
            .unwrap()
            .trim(),
        "3"
    );
    r.set_active_slot(0).unwrap();
    assert_eq!(slot_ring(d.path()).active_slot(), 0);
}

#[test]
fn an_out_of_range_active_slot_is_refused_and_a_bad_file_reads_as_the_first() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.set_active_slot(1).unwrap();
    r.set_active_slot(SLOT_COUNT).unwrap();
    assert_eq!(r.active_slot(), 1, "an out of range slot was stored");
    let file = d.path().join("States/GBA/mgba/Emerald/active-slot.txt");
    for junk in ["9\n", "0\n", "banana", ""] {
        std::fs::write(&file, junk).unwrap();
        assert_eq!(r.active_slot(), 0, "{junk:?}");
    }
}

#[test]
fn the_active_slot_is_not_listed_as_a_state_or_a_slot() {
    let d = tempdir().unwrap();
    let r = slot_ring(d.path());
    r.set_active_slot(2).unwrap();
    assert!(r.list().unwrap().is_empty());
    assert!(r.slots().iter().all(Option::is_none));
}
