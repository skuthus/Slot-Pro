use slot_store::GbFrame;
use slot_ui::{gbc_frame_face, OUT_H, OUT_W};

#[test]
fn every_shell_but_off_draws_a_whole_canvas_frame() {
    assert!(gbc_frame_face(GbFrame::Off).is_none());
    for frame in GbFrame::ALL.into_iter().skip(1) {
        let f = gbc_frame_face(frame).unwrap_or_else(|| panic!("{frame:?} drew nothing"));
        assert_eq!((f.w, f.h), (OUT_W, OUT_H));
        assert_eq!(f.rgba[3], 255, "{frame:?}: the corner is see-through");
    }
}
