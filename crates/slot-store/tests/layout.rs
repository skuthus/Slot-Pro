mod common;

use common::tmp_root;
use slot_store::{read_game_layout, write_game_layout, Layout, PadKey, Platform};

fn swapped() -> Layout {
    let mut l = Layout::default();
    l.set(PadKey::A, PadKey::B);
    l.set(PadKey::B, PadKey::A);
    l
}

#[test]
fn the_default_layout_sends_every_button_to_itself() {
    for k in PadKey::ALL {
        assert_eq!(Layout::default().sends(k), k);
    }
    assert_eq!(Layout::default().name(), "a,b,l,r,select,start,x,y");
}

#[test]
fn setting_one_button_changes_only_that_button() {
    let l = swapped();
    assert_eq!(l.sends(PadKey::A), PadKey::B);
    assert_eq!(l.sends(PadKey::B), PadKey::A);
    assert_eq!(l.sends(PadKey::L), PadKey::L);
    assert_eq!(l.name(), "b,a,l,r,select,start,x,y");
}

#[test]
fn a_layout_name_parses_back_to_the_same_layout() {
    let mut l = Layout::default();
    l.set(PadKey::Start, PadKey::Select);
    l.set(PadKey::L, PadKey::A);
    assert_eq!(Layout::parse(&l.name()), Some(l));
    assert_eq!(Layout::parse("  b , a,l,r,select,start\n"), Some(swapped()));
}

#[test]
fn a_layout_with_the_wrong_shape_or_an_unknown_key_does_not_parse() {
    for bad in [
        "",
        "a,b,l,r,select",
        "a,b,l,r,select,start,a",
        "a,b,l,r,select,start,x",
        "a,b,l,r,select,start,x,y,a",
        "a,b,l,r,select,z",
        "A,B,L,R,SELECT,START",
        "a b l r select start",
    ] {
        assert_eq!(Layout::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn stepping_a_key_wraps_around_both_ends() {
    assert_eq!(PadKey::Y.step(true), PadKey::A);
    assert_eq!(PadKey::A.step(false), PadKey::Y);
    assert_eq!(PadKey::Start.step(true), PadKey::X);
    assert_eq!(PadKey::A.step(true), PadKey::B);
}

#[test]
fn a_game_layout_is_written_under_its_platform_and_read_back() {
    let d = tmp_root();
    write_game_layout(d.path(), Platform::Gba, "Emerald", Some(swapped())).unwrap();
    let file = d.path().join("Config/Layouts/GBA/Emerald.txt");
    assert_eq!(
        std::fs::read_to_string(file).unwrap().trim(),
        "b,a,l,r,select,start,x,y"
    );
    assert_eq!(
        read_game_layout(d.path(), Platform::Gba, "Emerald"),
        Some(swapped())
    );
}

#[test]
fn a_game_layout_belongs_to_one_platform_and_one_game() {
    let d = tmp_root();
    write_game_layout(d.path(), Platform::Gba, "Emerald", Some(swapped())).unwrap();
    assert_eq!(read_game_layout(d.path(), Platform::Gb, "Emerald"), None);
    assert_eq!(read_game_layout(d.path(), Platform::Gba, "Ruby"), None);
}

#[test]
fn writing_none_removes_the_game_layout_and_is_harmless_when_there_is_none() {
    let d = tmp_root();
    write_game_layout(d.path(), Platform::Gba, "Emerald", None).unwrap();
    write_game_layout(d.path(), Platform::Gba, "Emerald", Some(swapped())).unwrap();
    write_game_layout(d.path(), Platform::Gba, "Emerald", None).unwrap();
    assert_eq!(read_game_layout(d.path(), Platform::Gba, "Emerald"), None);
    assert!(!d.path().join("Config/Layouts/GBA/Emerald.txt").exists());
}

#[test]
fn a_garbled_game_layout_file_reads_as_no_layout() {
    let d = tmp_root();
    let dir = d.path().join("Config/Layouts/GBA");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Emerald.txt"), "nonsense").unwrap();
    assert_eq!(read_game_layout(d.path(), Platform::Gba, "Emerald"), None);
}

#[test]
fn a_six_button_layout_from_before_x_and_y_reads_with_their_turbo_kept() {
    let l = Layout::parse("b,a,l,r,start,select").expect("an older layout");
    assert_eq!(l.sends(PadKey::A), PadKey::B);
    assert_eq!(l.sends(PadKey::Select), PadKey::Start);
    assert_eq!(l.sends(PadKey::X), PadKey::X);
    assert_eq!(l.sends(PadKey::Y), PadKey::Y);
}
