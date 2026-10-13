mod common;

use common::tmp_root;
use slot_store::{
    atomic_write, read_slot_state, write_slot_state, CartTitle, ColourCorrection, GbFrame,
    GbPalette, Layout, PadKey, Platform, Shader, SlotState, FF_SPEEDS, FF_SPEED_DEFAULT,
};
use tempfile::tempdir;

#[test]
fn atomic_write_leaves_no_partial_file_and_no_temp_behind() {
    let d = tempdir().unwrap();
    let p = d.path().join("x.bin");
    atomic_write(&p, b"first").unwrap();
    atomic_write(&p, &vec![7u8; 4_000_000]).unwrap();
    assert_eq!(std::fs::read(&p).unwrap().len(), 4_000_000);
    let strays: Vec<_> = std::fs::read_dir(d.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() != "x.bin")
        .collect();
    assert!(strays.is_empty(), "temp files left behind: {strays:?}");
}

#[test]
fn a_name_the_card_accepts_is_a_name_that_can_be_written() {
    let d = tempdir().unwrap();
    for len in [8usize, 100, 200, 239, 240, 245, 250, 251] {
        let p = d.path().join(format!("{}.sav", "a".repeat(len)));
        if std::fs::write(&p, b"control").is_err() {
            continue;
        }
        atomic_write(&p, b"save")
            .unwrap_or_else(|e| panic!("{len} character name: {e}, and a plain write took it"));
        assert_eq!(std::fs::read(&p).unwrap(), b"save");
    }
}

#[test]
fn a_long_multibyte_name_is_written_rather_than_panicking() {
    let d = tempdir().unwrap();
    for pad in 0..3 {
        let p = d
            .path()
            .join(format!("{}{}.sav", "x".repeat(pad), "ポ".repeat(82)));
        if std::fs::write(&p, b"control").is_err() {
            continue;
        }
        atomic_write(&p, b"save")
            .unwrap_or_else(|e| panic!("a Japanese cart name, pad {pad}: {e}"));
        assert_eq!(std::fs::read(&p).unwrap(), b"save");
    }
}

#[test]
fn corrupt_slot_state_reads_as_default_rather_than_panicking() {
    let d = tmp_root();
    std::fs::write(d.path().join("Config/slot.state"), b"\x00\xff not json").unwrap();
    assert_eq!(read_slot_state(d.path()), SlotState::default());
}

#[test]
fn slot_state_round_trips_including_a_stem_with_an_equals_sign() {
    let d = tmp_root();
    let s = SlotState {
        cart: Some("Cheats = On".into()),
        brightness: 3,
        blue_light: 9,
        volume: 71,
        muted: true,
        clock_set: true,
        utc_offset_min: 0,
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()), s);
}

#[test]
fn a_slot_state_missing_a_key_reads_as_default_not_half_populated() {
    let d = tmp_root();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\n",
    )
    .unwrap();
    assert_eq!(read_slot_state(d.path()), SlotState::default());
}

#[test]
fn an_out_of_range_level_reads_as_default() {
    let d = tmp_root();
    for body in [
        "cart=\nbrightness=10\nblue_light=1\nvolume=50\n",
        "cart=\nbrightness=3\nblue_light=10\nvolume=50\n",
        "cart=\nbrightness=3\nblue_light=1\nvolume=101\n",
    ] {
        std::fs::write(d.path().join("Config/slot.state"), body).unwrap();
        assert_eq!(
            read_slot_state(d.path()),
            SlotState::default(),
            "accepted {body:?}"
        );
    }
}

#[test]
fn a_first_boot_is_neither_dark_nor_silent() {
    let d = tmp_root();
    let s = read_slot_state(d.path());
    assert!(s.cart.is_none());
    assert!(s.brightness > 0, "boots with the backlight off");
    assert!(s.volume > 0, "boots muted");
}

#[test]
fn slot_state_round_trips_a_negative_utc_offset() {
    let d = tmp_root();
    let s = SlotState {
        cart: None,
        brightness: 5,
        blue_light: 0,
        volume: 60,
        muted: false,
        clock_set: true,
        utc_offset_min: -450,
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()).utc_offset_min, -450);
}

#[test]
fn a_line_the_reader_does_not_know_is_skipped() {
    let d = tmp_root();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nfrom_a_later_build=7\nblue_light=1\nvolume=40\nmuted=1\n\
         no equals sign at all\nclock_set=1\nutc_offset_min=-300\n",
    )
    .unwrap();
    let s = read_slot_state(d.path());
    assert_eq!(
        s.cart.as_deref(),
        Some("Emerald"),
        "the file was thrown away"
    );
    assert_eq!(
        (
            s.brightness,
            s.blue_light,
            s.volume,
            s.muted,
            s.clock_set,
            s.utc_offset_min
        ),
        (3, 1, 40, true, true, -300)
    );
}

#[test]
fn a_first_boot_rumbles_and_fast_forwards_silently_at_the_default() {
    let s = SlotState::default();
    assert!(s.rumble, "boots with the motor off");
    assert_eq!(s.ff_speed, FF_SPEED_DEFAULT);
    assert!(!s.ff_sound, "boots with fast forward audible");
    assert_eq!(
        s.colour_correction,
        ColourCorrection::Off,
        "boots with the picture tinted"
    );
    assert!(s.clock_24h, "boots with a 12 hour clock");
    assert_eq!(
        s.cart_title,
        CartTitle::Off,
        "boots with titles under the carts"
    );
    assert_eq!(s.shader_gba, Shader::Grid);
    assert_eq!(s.shader_gb, Shader::Simpletex);
}

#[test]
fn a_card_from_before_the_settings_keeps_all_its_values() {
    let d = tmp_root();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=1\nclock_set=1\nutc_offset_min=-300\n",
    )
    .unwrap();
    assert_eq!(
        read_slot_state(d.path()),
        SlotState {
            cart: Some("Emerald".into()),
            cart_platform: None,
            brightness: 3,
            blue_light: 1,
            volume: 40,
            volume_hp: 40,
            muted: true,
            muted_hp: true,
            clock_set: true,
            utc_offset_min: -300,
            rumble: true,
            ff_speed: FF_SPEED_DEFAULT,
            ff_sound: false,
            colour_correction: ColourCorrection::Off,
            shader_gba: Shader::Grid,
            shader_gb: Shader::Simpletex,
            eject_save: true,
            turbo: true,
            rewind: true,
            gb_palettes: false,
            gb_palette: GbPalette::DEFAULT,
            clock_24h: true,
            cart_title: CartTitle::Off,
            layout: Layout::default(),
            gb_frame: GbFrame::Off,
            analog: false,
            start_favorites: false,
            run_ahead: 0,
            lcd_ghosting: true,
        }
    );
}

#[test]
fn the_quick_menu_settings_round_trip_as_their_own_lines() {
    let d = tmp_root();
    let s = SlotState {
        clock_set: true,
        rumble: false,
        ff_speed: 2,
        ff_sound: true,
        colour_correction: ColourCorrection::GbAndGbc,
        shader_gba: Shader::Dot,
        shader_gb: Shader::Off,
        eject_save: false,
        turbo: false,
        rewind: false,
        gb_palettes: true,
        gb_palette: GbPalette::parse("GBC Dark Green →A").unwrap(),
        clock_24h: false,
        cart_title: CartTitle::Above,
        gb_frame: GbFrame::Teal,
        analog: true,
        layout: swapped_ab(),
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()), s);
    let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
    for line in [
        "rumble=0",
        "ff_speed=2",
        "ff_sound=1",
        "colour_correction=gb_gbc",
        "shader_gba=dot",
        "shader_gb=off",
        "eject_save=0",
        "turbo=0",
        "rewind=0",
        "gb_palettes=1",
        "gb_palette=GBC Dark Green →A",
        "clock_24h=0",
        "cart_title=above",
        "layout=b,a,l,r,select,start,x,y",
        "gb_frame=teal",
        "analog=1",
    ] {
        assert!(text.lines().any(|l| l == line), "no {line} in {text:?}");
    }
}

#[test]
fn every_speed_the_row_offers_round_trips_as_its_own_number() {
    for speed in FF_SPEEDS {
        let d = tmp_root();
        let s = SlotState {
            clock_set: true,
            ff_speed: speed,
            ..SlotState::default()
        };
        write_slot_state(d.path(), &s).unwrap();
        assert_eq!(read_slot_state(d.path()), s, "{speed}x did not come back");
        let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
        let want = format!("ff_speed={speed}");
        assert!(text.lines().any(|l| l == want), "no {want} in {text:?}");
    }
}

#[test]
fn an_older_build_reads_the_new_speed_as_its_own_default() {
    for speed in FF_SPEEDS.iter().filter(|&&n| n > 4) {
        assert!(
            !(2..=4).contains(speed),
            "{speed}x is inside the range an older build accepts, so it would read as a speed"
        );
    }
    assert_eq!(SlotState::default().ff_speed, FF_SPEED_DEFAULT);
    assert!(
        FF_SPEEDS.contains(&FF_SPEED_DEFAULT),
        "the default is not one of the speeds the row offers"
    );
}

#[test]
fn an_out_of_range_setting_falls_back_to_its_default() {
    let d = tmp_root();
    let known = "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=0\n";
    for bad in [
        "rumble=2\nff_speed=5\nff_sound=9\n",
        "rumble=\nff_speed=1\nff_sound=on\n",
        "rumble=-1\nff_speed=0\nff_sound=-1\n",
        "ff_speed=x\n",
        "colour_correction=2\n",
        "colour_correction=\n",
        "colour_correction=Auto\n",
        "ff_speed=5\n",
        "ff_speed=7\n",
        "ff_speed=8\n",
        "ff_speed=255\n",
    ] {
        std::fs::write(d.path().join("Config/slot.state"), format!("{known}{bad}")).unwrap();
        let s = read_slot_state(d.path());
        assert_eq!(
            (s.rumble, s.ff_speed, s.ff_sound, s.colour_correction),
            (true, FF_SPEED_DEFAULT, false, ColourCorrection::Off),
            "accepted {bad:?}"
        );
        assert_eq!(
            (s.cart.as_deref(), s.brightness, s.volume, s.clock_set),
            (Some("Emerald"), 3, 40, true),
            "{bad:?} took the rest of the card with it"
        );
    }
}

#[test]
fn every_platform_round_trips_as_its_own_line() {
    for platform in Platform::ALL {
        let d = tmp_root();
        let s = SlotState {
            cart: Some("Tetris".into()),
            cart_platform: Some(platform),
            clock_set: true,
            ..SlotState::default()
        };
        write_slot_state(d.path(), &s).unwrap();
        assert_eq!(
            read_slot_state(d.path()),
            s,
            "{platform:?} did not come back"
        );
        let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
        let want = format!("cart_platform={}", platform.dir_name().to_lowercase());
        assert!(text.lines().any(|l| l == want), "no {want} in {text:?}");
    }
}

#[test]
fn an_empty_slot_writes_an_empty_platform() {
    let d = tmp_root();
    let s = SlotState {
        clock_set: true,
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
    assert!(
        text.lines().any(|l| l == "cart_platform="),
        "no empty cart_platform line in {text:?}"
    );
    assert_eq!(read_slot_state(d.path()).cart_platform, None);
}

#[test]
fn an_unreadable_platform_reads_as_a_card_that_never_said() {
    let d = tmp_root();
    let known = "cart=Tetris\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=0\n";
    for (line, want) in [
        ("cart_platform=\n", None),
        ("cart_platform=nes\n", None),
        ("cart_platform=gameboy\n", None),
        ("cart_platform=0\n", None),
        ("cart_platform=nds\n", None),
        ("cart_platform=GBA\n", Some(Platform::Gba)),
        ("cart_platform=Gbc\n", Some(Platform::Gbc)),
    ] {
        std::fs::write(d.path().join("Config/slot.state"), format!("{known}{line}")).unwrap();
        let s = read_slot_state(d.path());
        assert_eq!(s.cart_platform, want, "read {line:?} wrong");
        assert_eq!(
            (s.cart.as_deref(), s.brightness, s.volume, s.clock_set),
            (Some("Tetris"), 3, 40, true),
            "{line:?} took the rest of the card with it"
        );
    }
}

#[test]
fn an_offset_outside_the_range_of_real_zones_reads_as_default() {
    let d = tmp_root();
    for body in [
        "cart=\nbrightness=5\nblue_light=0\nvolume=60\nmuted=0\nclock_set=1\nutc_offset_min=900\n",
        "cart=\nbrightness=5\nblue_light=0\nvolume=60\nmuted=0\nclock_set=1\nutc_offset_min=-780\n",
    ] {
        std::fs::write(d.path().join("Config/slot.state"), body).unwrap();
        assert_eq!(read_slot_state(d.path()), SlotState::default(), "{body}");
    }
}

#[test]
fn a_missing_or_unknown_palette_falls_back_to_dmg_green() {
    let d = tmp_root();
    let known = "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=0\n";
    for bad in [
        "",
        "gb_palette=\n",
        "gb_palette=Purple\n",
        "gb_palettes=2\ngb_palette=dmg green\n",
    ] {
        std::fs::write(d.path().join("Config/slot.state"), format!("{known}{bad}")).unwrap();
        let s = read_slot_state(d.path());
        assert_eq!(
            s.cart.as_deref(),
            Some("Emerald"),
            "{bad:?} lost the other settings"
        );
        assert!(!s.gb_palettes, "{bad:?} turned palettes on");
        assert_eq!(s.gb_palette, GbPalette::DEFAULT, "{bad:?}");
    }
}

#[test]
fn every_colour_correction_mode_round_trips_and_the_old_switch_still_reads() {
    for mode in ColourCorrection::ALL {
        let d = tmp_root();
        let s = SlotState {
            clock_set: true,
            colour_correction: mode,
            ..SlotState::default()
        };
        write_slot_state(d.path(), &s).unwrap();
        assert_eq!(read_slot_state(d.path()), s, "{mode:?} did not come back");
    }
    let known = "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=0\n";
    for (old, want) in [
        ("0", ColourCorrection::Off),
        ("1", ColourCorrection::All),
        ("on", ColourCorrection::All),
    ] {
        let d = tmp_root();
        std::fs::write(
            d.path().join("Config/slot.state"),
            format!("{known}colour_correction={old}\n"),
        )
        .unwrap();
        assert_eq!(read_slot_state(d.path()).colour_correction, want, "{old}");
    }
}

#[test]
fn each_colour_correction_mode_reaches_only_its_own_systems() {
    use ColourCorrection::*;
    let reach =
        |m: ColourCorrection| [Platform::Gba, Platform::Gb, Platform::Gbc].map(|p| m.applies_to(p));
    assert_eq!(reach(Off), [false, false, false]);
    assert_eq!(reach(GbaOnly), [true, false, false]);
    assert_eq!(reach(GbAndGbc), [false, true, true]);
    assert_eq!(reach(All), [true, true, true]);
}

fn swapped_ab() -> Layout {
    let mut l = Layout::default();
    l.set(PadKey::A, PadKey::B);
    l.set(PadKey::B, PadKey::A);
    l
}

#[test]
fn a_first_boot_sends_every_button_to_itself() {
    assert_eq!(SlotState::default().layout, Layout::default());
}

#[test]
fn a_remapped_layout_survives_a_restart() {
    let d = tmp_root();
    let s = SlotState {
        clock_set: true,
        layout: swapped_ab(),
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()).layout, swapped_ab());
}

#[test]
fn a_garbled_layout_line_falls_back_to_the_default_and_keeps_the_rest() {
    let d = tmp_root();
    let s = SlotState {
        brightness: 3,
        clock_set: true,
        layout: swapped_ab(),
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    let file = d.path().join("Config/slot.state");
    let text = std::fs::read_to_string(&file).unwrap().replace(
        "layout=b,a,l,r,select,start,x,y",
        "layout=a,b,zzz,r,select,start",
    );
    std::fs::write(&file, text).unwrap();
    let back = read_slot_state(d.path());
    assert_eq!(back.layout, Layout::default());
    assert_eq!(back.brightness, 3);
}

#[test]
fn every_gbc_frame_round_trips_and_a_stranger_reads_as_off() {
    for frame in GbFrame::ALL {
        let d = tmp_root();
        let s = SlotState {
            clock_set: true,
            gb_frame: frame,
            ..SlotState::default()
        };
        write_slot_state(d.path(), &s).unwrap();
        assert_eq!(read_slot_state(d.path()), s, "{frame:?} did not come back");
    }
    let d = tmp_root();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=0\ngb_frame=chartreuse\n",
    )
    .unwrap();
    assert_eq!(read_slot_state(d.path()).gb_frame, GbFrame::Off);
}

#[test]
fn analog_defaults_off_round_trips_and_reads_as_its_own_line() {
    let d = tmp_root();
    assert!(
        !read_slot_state(d.path()).analog,
        "a first boot is not analog"
    );
    write_slot_state(
        d.path(),
        &SlotState {
            analog: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(read_slot_state(d.path()).analog);
    let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
    assert!(text.lines().any(|l| l == "analog=1"), "{text}");
    write_slot_state(d.path(), &SlotState::default()).unwrap();
    let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
    assert!(text.lines().any(|l| l == "analog=0"), "{text}");
    assert!(!read_slot_state(d.path()).analog);
}

#[test]
fn a_slot_state_from_before_analog_reads_it_as_off() {
    let d = tmp_root();
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Emerald".into()),
            analog: true,
            ..Default::default()
        },
    )
    .unwrap();
    let path = d.path().join("Config/slot.state");
    let old: String = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("analog="))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, old).unwrap();
    let s = read_slot_state(d.path());
    assert_eq!(s.cart.as_deref(), Some("Emerald"));
    assert!(!s.analog);
}

#[test]
fn start_on_favorites_is_off_by_default_and_round_trips() {
    assert!(!SlotState::default().start_favorites);
    let d = tmp_root();
    let s = SlotState {
        clock_set: true,
        start_favorites: true,
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()), s);
    let text = std::fs::read_to_string(d.path().join("Config/slot.state")).unwrap();
    assert!(text.lines().any(|l| l == "start_favorites=1"), "{text}");
    write_slot_state(
        d.path(),
        &SlotState {
            start_favorites: false,
            ..s
        },
    )
    .unwrap();
    assert!(!read_slot_state(d.path()).start_favorites);
}

#[test]
fn a_state_file_from_before_favorites_starts_off_and_keeps_its_values() {
    let d = tmp_root();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=1\nclock_set=1\nutc_offset_min=-300\nanalog=1\n",
    )
    .unwrap();
    let s = read_slot_state(d.path());
    assert!(!s.start_favorites);
    assert!(s.analog);
    assert_eq!(s.cart.as_deref(), Some("Emerald"));
}
