use slot_store::{parse_stamp, FF_SPEEDS};
use slot_ui::{
    date_time_text, quick_caret_face, quick_label_face, quick_value_face, ClockPicker, QuickRow,
    QuickValue, UndoFace, MENU_PAD,
};

fn inked(f: &UndoFace, x: u32, y: u32) -> bool {
    f.rgba[((y * f.w + x) * 4 + 3) as usize] > 128
}

fn ink_columns(f: &UndoFace) -> (u32, u32) {
    let cols: Vec<u32> = (0..f.w)
        .filter(|&x| (0..f.h).any(|y| inked(f, x, y)))
        .collect();
    (
        *cols.first().expect("an empty face"),
        *cols.last().expect("an empty face"),
    )
}

#[test]
fn the_type_sits_exactly_menu_pad_in_from_both_sides_of_its_face() {
    for f in [
        quick_label_face(QuickRow::FastForwardSound),
        quick_label_face(QuickRow::Rumble),
        quick_value_face("Off", false),
        quick_value_face(QuickValue::Speed6.text(), true),
        quick_value_face("SEP 15 16:35", true),
    ] {
        let (first, last) = ink_columns(&f);
        let right = f.w - 1 - MENU_PAD;
        assert!(
            (MENU_PAD..MENU_PAD + 4).contains(&first),
            "ink starts at {first} of {}",
            f.w
        );
        assert!(
            (right - 4..=right).contains(&last),
            "ink ends at {last} of {}",
            f.w
        );
    }
}

#[test]
fn a_long_label_is_set_as_large_as_a_short_one() {
    let tall = |f: &UndoFace| {
        (0..f.h)
            .filter(|&y| (0..f.w).any(|x| inked(f, x, y)))
            .count()
    };
    let long = tall(&quick_label_face(QuickRow::FastForwardSound));
    let short = tall(&quick_label_face(QuickRow::Rumble));
    assert!(long + 1 >= short, "{long} rows of ink against {short}");
}

#[test]
fn the_fast_forward_row_offers_the_four_ceilings_the_card_can_hold() {
    assert_eq!(FF_SPEEDS, [2, 3, 4, 6]);
    assert_eq!(
        FF_SPEEDS.map(QuickValue::speed),
        [
            Some(QuickValue::Speed2),
            Some(QuickValue::Speed3),
            Some(QuickValue::Speed4),
            Some(QuickValue::Speed6),
        ]
    );
    for other in [1, 5, 7, 9, 16, 28, 255] {
        assert_eq!(
            QuickValue::speed(other),
            None,
            "{other}x is not a value the row has"
        );
    }
}

#[test]
fn the_rows_run_in_the_order_the_user_chose() {
    assert_eq!(
        QuickRow::MAIN.map(QuickRow::label),
        ["Screen", "Gameplay", "Controls", "Date & Time", "About"]
    );
    assert_eq!(
        QuickRow::SCREEN.map(QuickRow::label),
        [
            "GBA Shader",
            "GB / GBC Shader",
            "LCD Ghosting",
            "Color Correction",
            "GB Palettes",
            "GBC Frame",
            "Game Title",
            "Clock"
        ]
    );
    assert_eq!(
        QuickRow::GAME.map(QuickRow::label),
        [
            "Analog Mode",
            "Start on Favorites",
            "Run-Ahead",
            "Fast Forward",
            "Fast Forward Sound",
            "Rewind",
            "Turbo Buttons",
            "Rumble",
            "Auto Save on Eject"
        ]
    );
    assert_eq!(
        QuickRow::CONTROLS.map(QuickRow::label),
        [
            "A Button", "B Button", "L Button", "R Button", "Select", "Start", "X Button",
            "Y Button"
        ]
    );
    assert_eq!(
        QuickRow::PAUSE.map(QuickRow::label),
        [
            "Resume",
            "Save State",
            "Load State",
            "Shader",
            "Controls",
            "Reset Game",
            "Eject Game"
        ]
    );
    assert_eq!(
        QuickRow::GAME_CONTROLS.map(QuickRow::label),
        [
            "Layout", "A Button", "B Button", "L Button", "R Button", "Select", "Start",
            "X Button", "Y Button"
        ]
    );
    let opens: Vec<QuickRow> = QuickRow::ALL.into_iter().filter(|r| r.opens()).collect();
    for row in [
        QuickRow::Screen,
        QuickRow::Game,
        QuickRow::Controls,
        QuickRow::DateTime,
        QuickRow::About,
        QuickRow::GameControls,
        QuickRow::Resume,
        QuickRow::SaveState,
        QuickRow::LoadState,
        QuickRow::ResetGame,
        QuickRow::EjectGame,
    ] {
        assert!(opens.contains(&row), "{row:?} does not open");
    }
    assert_eq!(opens.len(), 11);
    for page in [
        &QuickRow::MAIN[..],
        &QuickRow::SCREEN[..],
        &QuickRow::GAME[..],
        &QuickRow::CONTROLS[..],
        &QuickRow::PAUSE[..],
        &QuickRow::GAME_CONTROLS[..],
    ] {
        for &row in page {
            assert_eq!(row.page(), page, "{row:?}");
        }
    }
}

#[test]
fn the_values_read_as_the_menu_prints_them() {
    assert_eq!(
        QuickValue::ALL.map(QuickValue::text),
        [
            "2×",
            "3×",
            "4×",
            "6×",
            "On",
            "Off",
            "LCD3x",
            "Grid",
            "Dot",
            "Simpletex",
            "Authentic GBC",
            "1 Frame",
            "2 Frames",
            "GBA Only",
            "GB / GBC",
            "All",
            "Below",
            "Above",
            "12 Hour",
            "24 Hour",
            "A",
            "B",
            "L",
            "R",
            "Select",
            "Start",
            "Turbo A",
            "Turbo B",
            "Berry",
            "Grape",
            "Kiwi",
            "Dandelion",
            "Teal",
            "Atomic Purple",
            "Black",
            "Global",
            "Custom"
        ]
    );
    assert_eq!(QuickValue::flag(true), QuickValue::On);
    assert_eq!(QuickValue::flag(false), QuickValue::Off);
}

#[test]
fn the_date_and_time_read_as_a_month_a_day_and_the_carousels_24_hour_clock() {
    let at = |stamp: &str, h24| date_time_text(parse_stamp(stamp).expect("a stamp"), h24);
    assert_eq!(at("2026-09-15_16-35-00", true), "SEP 15 16:35");
    assert_eq!(at("2027-01-05_04-07-59", true), "JAN 5 04:07");
    assert_eq!(at("2026-09-15_16-35-00", false), "SEP 15 4:35 PM");
    assert_eq!(at("2027-01-05_00-07-59", false), "JAN 5 12:07 AM");
    assert_eq!(at("2027-01-05_12-00-00", false), "JAN 5 12:00 PM");
}

#[test]
fn a_picker_for_a_set_clock_starts_at_the_local_time_and_its_offset() {
    let utc = parse_stamp("2026-09-15_21-35-42").expect("a stamp");
    let p = ClockPicker::local(utc, -300);
    assert_eq!(p.offset_min(), -300);
    assert_eq!(
        p.secs(),
        utc - 42,
        "the picker does not show the minute it was opened in"
    );
    assert!(p.text().starts_with("2026-09-15 16:35"), "{}", p.text());
}

#[test]
fn a_value_is_grey_until_its_row_is_in_hand() {
    let inkiest = |lit| {
        let f = quick_value_face("4×", lit);
        f.rgba
            .chunks(4)
            .max_by_key(|p| p[3])
            .map(|p| [p[0], p[1], p[2]])
            .expect("an empty face")
    };
    assert_eq!(inkiest(true), [0xf6, 0xf4, 0xef]);
    assert_eq!(inkiest(false), [0x9a, 0x9a, 0xa4]);
}

#[test]
fn the_arrows_are_faces_the_height_of_a_value() {
    for right in [false, true] {
        let caret = quick_caret_face(right);
        assert!(
            caret.w > 0 && caret.rgba.chunks(4).any(|p| p[3] > 0),
            "an empty arrow"
        );
        assert_eq!(caret.h, quick_value_face("On", true).h);
    }
}

#[test]
fn the_two_carets_point_different_ways() {
    assert_ne!(
        slot_ui::quick_caret_face(false).rgba,
        slot_ui::quick_caret_face(true).rgba,
        "a caret glyph is missing from the symbols font"
    );
}

#[test]
fn the_remap_rows_name_the_button_they_edit_and_the_values_the_one_they_send() {
    use slot_store::PadKey;
    for (row, key) in QuickRow::CONTROLS.into_iter().zip(PadKey::ALL) {
        assert_eq!(row.remaps(), Some(key));
        let shown = match key {
            PadKey::X => "turbo a",
            PadKey::Y => "turbo b",
            _ => key.name(),
        };
        assert_eq!(QuickValue::key(key).text().to_lowercase(), shown);
    }
    for (row, key) in QuickRow::GAME_CONTROLS[1..].iter().zip(PadKey::ALL) {
        assert_eq!(row.remaps(), Some(key));
    }
    assert_eq!(QuickRow::Resume.remaps(), None);
    assert_eq!(QuickRow::GameLayout.remaps(), None);
}

#[test]
fn the_pages_nest_and_back_out_to_the_row_that_opened_them() {
    assert_eq!(QuickRow::Controls.child(), Some(QuickRow::RemapA));
    assert_eq!(QuickRow::RemapStart.parent(), Some(QuickRow::Controls));
    assert_eq!(QuickRow::GameControls.child(), Some(QuickRow::GameLayout));
    assert_eq!(
        QuickRow::GameRemapStart.parent(),
        Some(QuickRow::GameControls)
    );
    assert_eq!(QuickRow::Resume.parent(), None);
}
