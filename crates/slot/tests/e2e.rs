mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use slot::app::Phase;
use slot::session::Session;
use slot_input::{Btn, Millis, RawEvent, POWER_HOLD_MS};
use slot_store::{read_slot_state, Core, Platform, StateRing};
use slot_ui::PowerChoice;

const FRAME_MS: Millis = 16;
const DT: f32 = 1.0 / 60.0;

struct Pass {
    session: Session,
    root: PathBuf,
    now: Millis,
}

impl Pass {
    fn boot(root: &Path) -> Self {
        Pass {
            session: Session::boot(root.to_path_buf()),
            root: root.to_path_buf(),
            now: 0,
        }
    }

    fn step(&mut self) {
        self.now += FRAME_MS;
        self.session.feed([], self.now);
        self.session.update(DT);
    }

    fn event(&mut self, ev: RawEvent) {
        self.now += FRAME_MS;
        self.session.feed([ev], self.now);
        self.session.update(DT);
    }

    fn tap(&mut self, b: Btn) {
        self.event(RawEvent::Down(b));
        self.event(RawEvent::Up(b));
    }

    fn hold(&mut self, b: Btn, ms: Millis) {
        self.event(RawEvent::Down(b));
        let until = self.now + ms;
        while self.now < until {
            self.step();
        }
    }

    fn chord(&mut self, b: Btn) {
        self.event(RawEvent::Down(Btn::Select));
        self.event(RawEvent::Down(b));
        self.event(RawEvent::Up(b));
        self.event(RawEvent::Up(Btn::Select));
    }

    fn until(&mut self, what: &str, cond: impl Fn(&Session) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !cond(&self.session) {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            self.step();
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn playing(&self) -> Option<&str> {
        match self.session.app().phase() {
            Phase::Playing { cart } => Some(cart),
            _ => None,
        }
    }

    fn frame(&self) -> Option<Vec<u8>> {
        self.session.frame().map(|f| f.to_vec())
    }

    fn expect_running(&mut self) {
        self.until("a first frame", |s| s.frame().is_some());
        let held = self.frame();
        self.until("the picture to move", move |s| {
            s.frame().map(|f| f.to_vec()) != held
        });
    }

    fn ring(&self, stem: &str) -> StateRing {
        StateRing::new(&self.root, Platform::Gba, Core::Mgba, stem)
    }
}

#[test]
fn the_whole_pass_from_boot_to_resume() {
    if let Some(dylib) = common::vendored_core() {
        std::env::set_var("SLOT_CORE", dylib);
    }
    let d = common::tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let root = d.path();
    let mut p = Pass::boot(root);

    assert!(matches!(p.session.app().phase(), Phase::SetClock { .. }));
    p.tap(Btn::A);

    assert!(matches!(p.session.app().phase(), Phase::Shelf));
    assert!(!p.session.has_core());

    p.tap(Btn::Right);
    p.tap(Btn::A);
    assert!(matches!(p.session.app().phase(), Phase::Inserting { .. }));
    p.until("the cart to seat", |s| {
        matches!(s.app().phase(), Phase::Playing { .. })
    });
    assert_eq!(p.playing(), Some("Emerald"));
    assert_eq!(read_slot_state(root).cart.as_deref(), Some("Emerald"));
    p.expect_running();

    p.chord(Btn::Up);
    p.chord(Btn::Right);
    p.tap(Btn::VolUp);
    p.session.app().flush_state();
    let levels = read_slot_state(root);
    assert_eq!(levels.brightness, 6);
    assert_eq!(levels.blue_light, 1);
    assert_eq!(levels.volume, 65);
    assert_eq!(p.session.app().blue_light(), 1);
    assert_eq!(
        p.playing(),
        Some("Emerald"),
        "a level adjustment paused the game"
    );

    p.chord(Btn::R1);
    let saved = p.ring("Emerald").slots();
    assert_eq!(saved.iter().flatten().count(), 1);
    let first = saved[0].as_ref().expect("the save missed the first slot");
    assert!(
        !std::fs::read(&first.thumb).expect("thumb").is_empty(),
        "the slot has no picture"
    );

    p.event(RawEvent::Down(Btn::R2));
    p.expect_running();
    p.event(RawEvent::Up(Btn::R2));
    p.event(RawEvent::Down(Btn::L2));
    for _ in 0..10 {
        p.step();
        std::thread::sleep(Duration::from_millis(2));
    }
    p.event(RawEvent::Up(Btn::L2));
    p.expect_running();

    p.tap(Btn::Menu);
    p.event(RawEvent::Down(Btn::Menu));
    assert!(matches!(p.session.app().phase(), Phase::Polaroids { .. }));
    let shown = p.session.app().polaroid_entries();
    assert_eq!(shown.len(), 3);
    assert!(shown[0].is_some() && shown[1].is_none() && shown[2].is_none());
    p.event(RawEvent::Up(Btn::Menu));
    p.tap(Btn::A);
    assert_eq!(p.playing(), Some("Emerald"));
    p.expect_running();

    p.event(RawEvent::Down(Btn::Menu));
    p.until("the eject", |s| {
        !matches!(s.app().phase(), Phase::Playing { .. })
    });
    p.event(RawEvent::Up(Btn::Menu));
    assert!(p
        .ring("Emerald")
        .read_resume()
        .expect("read resume")
        .is_some());
    assert_eq!(read_slot_state(root).cart, None);
    p.until("the shelf", |s| matches!(s.app().phase(), Phase::Shelf));
    assert!(!p.session.has_core(), "the core outlived the cart");

    p.tap(Btn::A);
    p.until("the cart to seat again", |s| {
        matches!(s.app().phase(), Phase::Playing { .. })
    });
    assert_eq!(p.playing(), Some("Emerald"));

    p.tap(Btn::Power);
    assert_eq!(
        p.session.app().power_menu(),
        Some(PowerChoice::Sleep.index()),
        "a power tap opens the menu on Sleep"
    );
    p.tap(Btn::A);
    assert!(matches!(p.session.app().phase(), Phase::Doze { .. }));
    assert!(!p.session.app().powering_off());
    p.tap(Btn::Power);
    assert_eq!(p.playing(), Some("Emerald"));

    p.hold(Btn::Power, POWER_HOLD_MS + FRAME_MS);
    assert!(p.session.app().powering_off(), "the hold is a shutdown");
    drop(p);
    let mut p = Pass::boot(root);
    assert!(
        matches!(p.session.app().phase(), Phase::Inserting { .. }),
        "boot showed the shelf with a cart still in the slot"
    );
    p.until("the resumed cart", |s| {
        matches!(s.app().phase(), Phase::Playing { .. })
    });
    assert_eq!(p.playing(), Some("Emerald"));
    p.expect_running();
}
