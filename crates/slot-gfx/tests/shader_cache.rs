use slot_gfx::{set_shader_cache, Compositor, HeadlessSurface};

#[test]
fn a_second_compositor_comes_up_from_the_kept_binaries() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let dir = tempfile::tempdir().expect("a temp dir");
    set_shader_cache(dir.path().join("shaders"));
    let first = Compositor::new(&surface).expect("a compositor from source");
    drop(first);
    let kept = std::fs::read_dir(dir.path().join("shaders"))
        .map(|d| d.count())
        .unwrap_or(0);
    if kept == 0 {
        eprintln!("this driver offers no program binaries, nothing to check");
        return;
    }
    let again = Compositor::new(&surface).expect("a compositor from the kept binaries");
    drop(again);
    let after = std::fs::read_dir(dir.path().join("shaders"))
        .unwrap()
        .count();
    assert_eq!(after, kept, "a kept binary was refused and thrown away");
}
