mod common;

use common::tmp_root;
use slot_store::{read_favorites, write_favorites, Platform};

fn fav(platform: Platform, stem: &str) -> (Platform, String) {
    (platform, stem.to_string())
}

#[test]
fn a_card_with_no_favorites_file_has_no_favorites() {
    let d = tmp_root();
    assert!(read_favorites(d.path()).is_empty());
}

#[test]
fn favorites_round_trip_in_order_as_platform_slash_stem_lines() {
    let d = tmp_root();
    let favs = vec![
        fav(Platform::Gba, "Fusion"),
        fav(Platform::Gb, "Tetris"),
        fav(Platform::Gbc, "Crystal"),
        fav(Platform::Gba, "Zelda: The Minish Cap"),
    ];
    write_favorites(d.path(), &favs).unwrap();
    assert_eq!(read_favorites(d.path()), favs);
    let text = std::fs::read_to_string(d.path().join("Config/favorites.txt")).unwrap();
    assert_eq!(
        text,
        "GBA/Fusion\nGB/Tetris\nGBC/Crystal\nGBA/Zelda: The Minish Cap\n"
    );
}

#[test]
fn writing_none_leaves_an_empty_list_that_reads_back_empty() {
    let d = tmp_root();
    write_favorites(d.path(), &[fav(Platform::Gba, "Fusion")]).unwrap();
    write_favorites(d.path(), &[]).unwrap();
    assert!(read_favorites(d.path()).is_empty());
}

#[test]
fn junk_lines_are_skipped_and_the_good_ones_kept() {
    let d = tmp_root();
    std::fs::write(
        d.path().join("Config/favorites.txt"),
        "GBA/Fusion\n\nnonsense\nN64/Mario\nGB/\n/Orphan\nGBC/Crystal\r\ngba/Emerald\n",
    )
    .unwrap();
    assert_eq!(
        read_favorites(d.path()),
        vec![
            fav(Platform::Gba, "Fusion"),
            fav(Platform::Gbc, "Crystal"),
            fav(Platform::Gba, "Emerald"),
        ]
    );
}

#[test]
fn a_stem_with_a_slash_in_the_name_stays_whole() {
    let d = tmp_root();
    std::fs::write(d.path().join("Config/favorites.txt"), "GB/Tom/Jerry\n").unwrap();
    assert_eq!(
        read_favorites(d.path()),
        vec![fav(Platform::Gb, "Tom/Jerry")]
    );
}
