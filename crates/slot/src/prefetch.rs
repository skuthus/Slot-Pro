use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use slot_input::Millis;

pub const PREFETCH_REST_MS: Millis = 500;

const CHUNK: usize = 1 << 20;

#[derive(Default)]
pub struct Prefetch {
    resting: Option<(PathBuf, Millis)>,
    read: Option<PathBuf>,
    turn: Arc<AtomicU64>,
    worker: Option<JoinHandle<u64>>,
}

impl Prefetch {
    pub fn browse(&mut self, rom: Option<&Path>, now: Millis) {
        let Some(rom) = rom else {
            self.resting = None;
            self.read = None;
            self.turn.fetch_add(1, Ordering::Relaxed);
            return;
        };
        let since = match &self.resting {
            Some((at, since)) if at == rom => *since,
            _ => {
                self.resting = Some((rom.to_path_buf(), now));
                self.read = None;
                self.turn.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };
        if now.saturating_sub(since) < PREFETCH_REST_MS || self.read.as_deref() == Some(rom) {
            return;
        }
        self.read = Some(rom.to_path_buf());
        let turn = self.turn.clone();
        let mine = turn.fetch_add(1, Ordering::Relaxed) + 1;
        let path = rom.to_path_buf();
        self.worker = std::thread::Builder::new()
            .name("slot-prefetch".into())
            .spawn(move || read_through(&path, &turn, mine))
            .ok();
    }

    pub fn reading(&self) -> Option<&Path> {
        self.read.as_deref()
    }

    pub fn wait(&mut self) -> Option<u64> {
        self.worker.take().and_then(|w| w.join().ok())
    }
}

fn read_through(path: &Path, turn: &AtomicU64, mine: u64) -> u64 {
    let start = Instant::now();
    let Ok(mut file) = std::fs::File::open(path) else {
        return 0;
    };
    let mut buf = vec![0u8; CHUNK];
    let mut total = 0u64;
    while turn.load(Ordering::Relaxed) == mine {
        match file.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => total += n as u64,
        }
    }
    eprintln!(
        "slot: prefetch: {} bytes of {} in {:.1} ms",
        total,
        path.display(),
        start.elapsed().as_secs_f64() * 1e3
    );
    total
}

pub fn warm_resume(root: &Path) {
    let root = root.to_path_buf();
    let _ = std::thread::Builder::new()
        .name("slot-warm".into())
        .spawn(move || {
            let start = Instant::now();
            let files = resume_files(&root);
            let total: u64 = files.iter().map(|f| read_all(f)).sum();
            if total > 0 {
                eprintln!(
                    "slot: warm: {total} bytes over {} files in {:.1} ms",
                    files.len(),
                    start.elapsed().as_secs_f64() * 1e3
                );
            }
        });
}

fn resume_files(root: &Path) -> Vec<PathBuf> {
    let state = slot_store::read_slot_state(root);
    let (Some(stem), Some(platform)) = (state.cart, state.cart_platform) else {
        return Vec::new();
    };
    if !state.clock_set {
        return Vec::new();
    }
    let games = root.join("Games").join(platform.dir_name());
    let Some(rom) = platform
        .extensions()
        .iter()
        .map(|ext| games.join(format!("{stem}.{ext}")))
        .find(|p| p.exists())
    else {
        return Vec::new();
    };
    let core = slot_store::core_for_platform(root, &stem, platform);
    let mut files = vec![rom];
    files.extend(
        crate::core::candidates(root, core)
            .into_iter()
            .find(|p| p.exists()),
    );
    files.push(
        root.join("States")
            .join(platform.dir_name())
            .join(core.as_str())
            .join(&stem)
            .join("resume.state"),
    );
    files.push(crate::persist::sav_path(root, platform, &stem));
    files
}

fn read_all(path: &Path) -> u64 {
    let Ok(mut file) = std::fs::File::open(path) else {
        return 0;
    };
    let mut buf = vec![0u8; CHUNK];
    let mut total = 0u64;
    while let Ok(n @ 1..) = file.read(&mut buf) {
        total += n as u64;
    }
    total
}
