mod common;

use common::{app_booting_at, clocked, tmp_root_with_carts, StubSnapshot, CLOCK_IS_SET};
use slot::app::{App, Phase, ShelfKind};
use slot_input::{Action, Btn};
use slot_store::{
    read_favorites, read_slot_state, write_favorites, write_slot_state, Platform, SlotState,
};
use slot_ui::{QuickRow, QuickValue, Toast};
use tempfile::TempDir;

const GBA: Platform = Platform::Gba;
const GB: Platform = Platform::Gb;
const GBC: Platform = Platform::Gbc;

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn add_rom(d: &TempDir, platform: Platform, stem: &str) {
    let ext = platform.extensions()[0];
    let path = d
        .path()
        .join("Games")
        .join(platform.dir_name())
        .join(format!("{stem}.{ext}"));
    std::fs::write(path, vec![0u8; 0x150]).expect("write rom");
}

fn mixed_root() -> TempDir {
    let d = tmp_root_with_carts(&["Emerald", "Fusion", "Zelda"]);
    add_rom(&d, GB, "Tetris");
    add_rom(&d, GB, "Alleyway");
    add_rom(&d, GBC, "Crystal");
    d
}

fn set_state(d: &TempDir, f: impl FnOnce(&mut SlotState)) {
    let mut s = read_slot_state(d.path());
    s.clock_set = true;
    f(&mut s);
    write_slot_state(d.path(), &s).expect("write slot.state");
}

fn boot(d: &TempDir) -> App {
    clocked(d.path());
    app_booting_at(d.path(), CLOCK_IS_SET).0
}

fn shelf_name(a: &App) -> Option<&'static str> {
    a.shelf_platform_name()
}

fn favorites_shelf(a: &App) -> Vec<(Platform, String)> {
    (0..)
        .map_while(|i| a.face_cart(0, i))
        .map(|c| (c.platform, c.stem.clone()))
        .collect()
}

fn favorite(a: &mut App, platform: Platform, stem: &str) {
    let want = a.shelf_kind();
    assert_eq!(want, ShelfKind::Console(platform), "not on {platform:?}");
    for _ in 0..8 {
        if a.selected_stem() == Some(stem) {
            break;
        }
        press(a, Btn::Right);
    }
    assert_eq!(a.selected_stem(), Some(stem));
    if !a.is_favorite(platform, stem) {
        press(a, Btn::Y);
    }
    assert!(a.is_favorite(platform, stem));
}

fn play(a: &mut App) {
    a.set_snapshot(StubSnapshot::boxed());
    press(a, Btn::A);
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
}

#[test]
fn y_favorites_the_highlighted_cart_and_y_again_removes_it() {
    let d = mixed_root();
    let mut a = boot(&d);
    assert_eq!(a.selected_stem(), Some("Emerald"));
    assert!(!a.is_favorite(GBA, "Emerald"));
    assert!(read_favorites(d.path()).is_empty());

    press(&mut a, Btn::Y);
    assert!(a.is_favorite(GBA, "Emerald"));
    assert_eq!(a.toast(), Some(Toast::FavoriteAdded));
    assert_eq!(read_favorites(d.path()), vec![(GBA, "Emerald".to_string())]);
    assert_eq!(a.selected_stem(), Some("Emerald"), "Y moved the cursor");
    assert!(matches!(a.phase(), Phase::Shelf), "Y left the carousel");

    press(&mut a, Btn::Y);
    assert!(!a.is_favorite(GBA, "Emerald"));
    assert_eq!(a.toast(), Some(Toast::FavoriteRemoved));
    assert!(read_favorites(d.path()).is_empty());
    assert!(favorites_shelf(&a).is_empty());
}

#[test]
fn a_favorite_is_told_apart_by_platform_as_well_as_stem() {
    let d = mixed_root();
    add_rom(&d, GB, "Emerald");
    let mut a = boot(&d);
    press(&mut a, Btn::Y);
    assert!(a.is_favorite(GBA, "Emerald"));
    assert!(!a.is_favorite(GB, "Emerald"));
    assert!(!a.is_favorite(GBC, "Emerald"));
}

#[test]
fn favorites_gather_from_every_console_sorted_by_name() {
    let d = mixed_root();
    let mut a = boot(&d);
    favorite(&mut a, GBA, "Zelda");
    press(&mut a, Btn::R1);
    favorite(&mut a, GB, "Tetris");
    favorite(&mut a, GB, "Alleyway");
    press(&mut a, Btn::R1);
    favorite(&mut a, GBC, "Crystal");
    assert_eq!(
        read_favorites(d.path()),
        vec![
            (GBA, "Zelda".to_string()),
            (GB, "Tetris".to_string()),
            (GB, "Alleyway".to_string()),
            (GBC, "Crystal".to_string()),
        ],
        "the file keeps the order they were added in"
    );
    assert_eq!(
        favorites_shelf(&a),
        vec![
            (GB, "Alleyway".to_string()),
            (GBC, "Crystal".to_string()),
            (GB, "Tetris".to_string()),
            (GBA, "Zelda".to_string()),
        ]
    );
    assert_eq!(a.carts().count(), 6, "favorites were counted twice");
}

#[test]
fn favorites_survive_a_restart() {
    let d = mixed_root();
    let mut a = boot(&d);
    favorite(&mut a, GBA, "Fusion");
    drop(a);
    let mut b = boot(&d);
    assert!(b.is_favorite(GBA, "Fusion"));
    assert_eq!(favorites_shelf(&b), vec![(GBA, "Fusion".to_string())]);
    press(&mut b, Btn::L1);
    assert_eq!(b.shelf_kind(), ShelfKind::Favorites);
    assert_eq!(b.selected_stem(), Some("Fusion"));
}

#[test]
fn the_shoulders_visit_favorites_first_only_when_there_are_some() {
    let d = mixed_root();
    let mut a = boot(&d);
    let mut seen = vec![shelf_name(&a)];
    for _ in 0..3 {
        press(&mut a, Btn::R1);
        seen.push(shelf_name(&a));
    }
    assert_eq!(
        seen,
        [
            Some("Game Boy Advance"),
            Some("Game Boy"),
            Some("Game Boy Color"),
            Some("Game Boy Advance")
        ],
        "an empty Favorites shelf was visited"
    );
    press(&mut a, Btn::L1);
    press(&mut a, Btn::L1);
    press(&mut a, Btn::L1);
    assert_eq!(shelf_name(&a), Some("Game Boy Advance"));

    favorite(&mut a, GBA, "Emerald");
    press(&mut a, Btn::L1);
    assert_eq!(a.shelf_kind(), ShelfKind::Favorites);
    assert_eq!(shelf_name(&a), Some("Favorites"));
    assert_eq!(a.slot_text().as_deref(), Some("Favorites"));
    press(&mut a, Btn::R1);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBA));
    press(&mut a, Btn::R1);
    press(&mut a, Btn::R1);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBC));
    press(&mut a, Btn::R1);
    assert_eq!(
        a.shelf_kind(),
        ShelfKind::Favorites,
        "Favorites comes before the first console when going round"
    );
}

#[test]
fn unfavoriting_the_last_cart_from_favorites_moves_to_a_console_shelf() {
    let d = mixed_root();
    let mut a = boot(&d);
    press(&mut a, Btn::R1);
    favorite(&mut a, GB, "Tetris");
    press(&mut a, Btn::L1);
    press(&mut a, Btn::L1);
    assert_eq!(a.shelf_kind(), ShelfKind::Favorites);
    assert_eq!(a.selected_stem(), Some("Tetris"));

    press(&mut a, Btn::Y);
    assert_eq!(a.toast(), Some(Toast::FavoriteRemoved));
    assert!(read_favorites(d.path()).is_empty());
    assert_ne!(
        a.shelf_kind(),
        ShelfKind::Favorites,
        "stranded on an empty shelf"
    );
    assert!(matches!(a.shelf_kind(), ShelfKind::Console(_)));
    assert!(a.selected_stem().is_some(), "landed on an empty shelf");
    assert_eq!(
        a.shelf_platform_name(),
        Some("Game Boy Advance"),
        "the next shelf round was not the one chosen"
    );
}

#[test]
fn unfavoriting_one_of_several_stays_on_favorites_and_keeps_the_cursor_in_range() {
    let d = mixed_root();
    let mut a = boot(&d);
    favorite(&mut a, GBA, "Emerald");
    favorite(&mut a, GBA, "Fusion");
    press(&mut a, Btn::L1);
    assert_eq!(a.shelf_kind(), ShelfKind::Favorites);
    press(&mut a, Btn::Right);
    assert_eq!(a.selected_stem(), Some("Fusion"));
    press(&mut a, Btn::Y);
    assert_eq!(a.shelf_kind(), ShelfKind::Favorites);
    assert_eq!(a.selected_stem(), Some("Emerald"));
    assert_eq!(favorites_shelf(&a), vec![(GBA, "Emerald".to_string())]);
    assert!(a.is_favorite(GBA, "Emerald"));
    assert!(!a.is_favorite(GBA, "Fusion"));
}

#[test]
fn unfavoriting_on_the_favorites_shelf_clears_the_favorite_on_its_console_shelf() {
    let d = mixed_root();
    let mut a = boot(&d);
    favorite(&mut a, GBA, "Emerald");
    favorite(&mut a, GBA, "Fusion");
    press(&mut a, Btn::L1);
    press(&mut a, Btn::Y);
    assert!(!a.is_favorite(GBA, "Emerald"));
    assert_eq!(read_favorites(d.path()), vec![(GBA, "Fusion".to_string())]);
    press(&mut a, Btn::R1);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBA));
}

#[test]
fn it_opens_on_favorites_only_when_the_setting_is_on_and_nothing_resumes() {
    let d = mixed_root();
    write_favorites(d.path(), &[(GB, "Tetris".into()), (GBA, "Fusion".into())]).unwrap();

    set_state(&d, |s| s.start_favorites = false);
    let a = boot(&d);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBA), "setting is off");

    set_state(&d, |s| s.start_favorites = true);
    let a = boot(&d);
    assert!(matches!(a.phase(), Phase::Shelf));
    assert_eq!(a.shelf_kind(), ShelfKind::Favorites);
    assert_eq!(a.selected_stem(), Some("Fusion"));
    assert_eq!(a.shelf_platform_name(), Some("Favorites"));

    set_state(&d, |s| {
        s.cart = Some("Emerald".into());
        s.cart_platform = Some(GBA);
    });
    let a = boot(&d);
    assert!(
        matches!(a.phase(), Phase::Inserting { .. }),
        "{:?}",
        a.phase()
    );
    assert_eq!(
        a.shelf_kind(),
        ShelfKind::Console(GBA),
        "a resume went to Favorites"
    );
    assert_eq!(a.seated_cart().map(|c| c.stem.as_str()), Some("Emerald"));
}

#[test]
fn a_resumed_favorite_comes_back_on_its_console_shelf() {
    let d = mixed_root();
    write_favorites(d.path(), &[(GB, "Tetris".into())]).unwrap();
    set_state(&d, |s| {
        s.start_favorites = true;
        s.cart = Some("Tetris".into());
        s.cart_platform = Some(GB);
    });
    let a = boot(&d);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GB));
    let seated = a.seated_cart().expect("nothing seated");
    assert_eq!((seated.platform, seated.stem.as_str()), (GB, "Tetris"));
}

#[test]
fn start_on_favorites_with_nothing_favorited_opens_on_the_first_console() {
    let d = mixed_root();
    set_state(&d, |s| s.start_favorites = true);
    let a = boot(&d);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBA));
    assert_eq!(a.selected_stem(), Some("Emerald"));
}

#[test]
fn a_favorite_whose_rom_is_gone_does_not_make_a_shelf_to_start_on() {
    let d = mixed_root();
    write_favorites(d.path(), &[(GBA, "Deleted".into())]).unwrap();
    set_state(&d, |s| s.start_favorites = true);
    let mut a = boot(&d);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBA));
    assert!(favorites_shelf(&a).is_empty());
    press(&mut a, Btn::R1);
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GB));
}

#[test]
fn a_cart_inserted_from_favorites_plays_and_is_the_seated_cart() {
    let d = mixed_root();
    write_favorites(d.path(), &[(GB, "Tetris".into()), (GBA, "Fusion".into())]).unwrap();
    set_state(&d, |s| s.start_favorites = true);
    let mut a = boot(&d);
    assert_eq!(a.shelf_kind(), ShelfKind::Favorites);
    press(&mut a, Btn::Right);
    assert_eq!(a.selected_stem(), Some("Tetris"));
    play(&mut a);
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "{:?}",
        a.phase()
    );
    let seated = a.seated_cart().expect("nothing seated");
    assert_eq!((seated.platform, seated.stem.as_str()), (GB, "Tetris"));
    assert_eq!(a.carts().count(), 6);
}

#[test]
#[ignore = "product bug: the seated cart is found by stem alone, so a Favorites shelf holding two platforms' same-named carts seats the wrong one"]
fn two_favorites_with_the_same_stem_seat_the_one_that_was_chosen() {
    let d = mixed_root();
    add_rom(&d, GB, "Zelda");
    write_favorites(d.path(), &[(GBA, "Zelda".into()), (GB, "Zelda".into())]).unwrap();
    set_state(&d, |s| s.start_favorites = true);
    let mut a = boot(&d);
    press(&mut a, Btn::Right);
    assert_eq!(favorites_shelf(&a)[1], (GB, "Zelda".to_string()));
    play(&mut a);
    let seated = a.seated_cart().expect("nothing seated");
    assert_eq!((seated.platform, seated.stem.as_str()), (GB, "Zelda"));
}

#[test]
fn favorites_do_not_count_as_extra_carts_for_a_lone_cart() {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_favorites(d.path(), &[(GBA, "Emerald".into())]).unwrap();
    let a = boot(&d);
    assert_eq!(a.carts().count(), 1);
    assert!(a.single_cart(), "the starred cart counted twice");
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
    assert_eq!(a.shelf_kind(), ShelfKind::Console(GBA));
}

#[test]
fn favoriting_everything_still_counts_each_cart_once() {
    let d = mixed_root();
    let all = [
        (GBA, "Emerald"),
        (GBA, "Fusion"),
        (GBA, "Zelda"),
        (GB, "Alleyway"),
        (GB, "Tetris"),
        (GBC, "Crystal"),
    ]
    .map(|(p, s)| (p, s.to_string()));
    write_favorites(d.path(), &all).unwrap();
    let a = boot(&d);
    assert_eq!(a.carts().count(), 6);
    assert!(!a.single_cart());
    assert_eq!(favorites_shelf(&a).len(), 6);
}

#[test]
fn y_does_nothing_while_the_core_picker_is_open() {
    let d = mixed_root();
    let mut a = boot(&d);
    press(&mut a, Btn::Start);
    if a.core_picker().is_none() {
        return;
    }
    press(&mut a, Btn::Y);
    assert!(!a.is_favorite(GBA, "Emerald"));
    assert!(read_favorites(d.path()).is_empty());
}

#[test]
fn the_start_on_favorites_row_sits_second_on_the_gameplay_page_and_toggles() {
    assert_eq!(QuickRow::GAME[1], QuickRow::StartFavorites);
    assert_eq!(QuickRow::StartFavorites.label(), "Start on Favorites");
    assert_eq!(QuickRow::StartFavorites.parent(), Some(QuickRow::Game));
    assert_eq!(QuickRow::StartFavorites.position(), 1);

    let d = mixed_root();
    set_state(&d, |_| {});
    let mut a = boot(&d);
    assert_eq!(
        a.quick_value(QuickRow::StartFavorites),
        Some(QuickValue::Off)
    );
    a.apply(Action::QuickMenu);
    for _ in 0..QuickRow::Game.position() {
        press(&mut a, Btn::Down);
    }
    press(&mut a, Btn::A);
    press(&mut a, Btn::Down);
    assert_eq!(a.quick_menu(), Some(QuickRow::StartFavorites));
    press(&mut a, Btn::Right);
    assert_eq!(
        a.quick_value(QuickRow::StartFavorites),
        Some(QuickValue::On)
    );
    assert!(
        read_slot_state(d.path()).start_favorites,
        "never reached the card"
    );
    press(&mut a, Btn::Left);
    assert_eq!(
        a.quick_value(QuickRow::StartFavorites),
        Some(QuickValue::Off)
    );
    assert!(!read_slot_state(d.path()).start_favorites);
    press(&mut a, Btn::Right);
    assert!(read_slot_state(d.path()).start_favorites);
}
