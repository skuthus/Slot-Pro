use std::path::{Path, PathBuf};

use crate::atomic::{atomic_write, sync_dir};
use crate::core::Core;
use crate::platform::Platform;

pub const RING_MAX: usize = 10;
pub const SLOT_COUNT: usize = 3;

const STATE_EXT: &str = "state";
const THUMB_EXT: &str = "png";
const RESUME: &str = "resume";
const PREVIOUS: &str = "resume-prev";
const REFUSED: &str = "resume-refused";
const ACTIVE_SLOT: &str = "active-slot.txt";

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StateEntry {
    pub stamp: String,
    pub state: PathBuf,
    pub thumb: PathBuf,
}

#[derive(Clone)]
pub struct StateRing {
    dir: PathBuf,
}

impl StateRing {
    pub fn new(root: &Path, platform: Platform, core: Core, stem: &str) -> Self {
        StateRing {
            dir: root
                .join("States")
                .join(platform.dir_name())
                .join(core.as_str())
                .join(stem),
        }
    }

    pub fn push(&self, state: &[u8], thumb_png: &[u8], stamp: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        atomic_write(&self.path(stamp, THUMB_EXT), thumb_png)?;
        atomic_write(&self.path(stamp, STATE_EXT), state)?;
        self.evict()
    }

    pub fn list(&self) -> std::io::Result<Vec<StateEntry>> {
        let dir = match std::fs::read_dir(&self.dir) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };

        let mut entries = Vec::new();
        for entry in dir {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some(STATE_EXT) {
                continue;
            }
            let Some(stamp) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !is_stamp(stamp) {
                continue;
            }
            entries.push(StateEntry {
                thumb: self.path(stamp, THUMB_EXT),
                state: self.path(stamp, STATE_EXT),
                stamp: stamp.to_string(),
            });
        }
        entries.sort_by(|a, b| b.stamp.cmp(&a.stamp));
        Ok(entries)
    }

    pub fn read(&self, stamp: &str) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
        let state = std::fs::read(self.stamped(stamp, STATE_EXT)?)?;
        let thumb = std::fs::read(self.path(stamp, THUMB_EXT)).unwrap_or_default();
        Ok((state, thumb))
    }

    pub fn remove(&self, stamp: &str) -> std::io::Result<()> {
        std::fs::remove_file(self.stamped(stamp, STATE_EXT)?)?;
        let _ = std::fs::remove_file(self.path(stamp, THUMB_EXT));
        Ok(())
    }

    /// Writes the resume state, keeping the one it replaces as `resume-prev` so a write the card
    /// loses costs minutes of play rather than everything since the last in-game save.
    pub fn write_resume(&self, state: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let resume = self.path(RESUME, STATE_EXT);
        if std::fs::metadata(&resume).is_ok_and(|m| m.len() > 0) {
            std::fs::rename(&resume, self.path(PREVIOUS, STATE_EXT))?;
        }
        atomic_write(&resume, state)
    }

    /// The resume state, or the one before it when the card has lost or emptied it.
    pub fn read_resume(&self) -> std::io::Result<Option<Vec<u8>>> {
        for name in [RESUME, PREVIOUS] {
            match std::fs::read(self.path(name, STATE_EXT)) {
                Ok(b) if !b.is_empty() => return Ok(Some(b)),
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
        Ok(None)
    }

    pub fn retire_resume(&self, stamp: &str) -> std::io::Result<Option<PathBuf>> {
        let from = self.path(RESUME, STATE_EXT);
        if !from.exists() {
            return Ok(None);
        }
        let to = self.free_refused(stamp);
        std::fs::rename(&from, &to)?;
        sync_dir(&to);
        Ok(Some(to))
    }

    fn free_refused(&self, stamp: &str) -> PathBuf {
        let base = format!("{REFUSED}-{stamp}");
        let first = self.path(&base, STATE_EXT);
        if !first.exists() {
            return first;
        }
        (2u32..)
            .map(|n| self.path(&format!("{base}-{n}"), STATE_EXT))
            .find(|p| !p.exists())
            .unwrap_or(first)
    }

    fn evict(&self) -> std::io::Result<()> {
        for old in self.list()?.into_iter().skip(RING_MAX) {
            std::fs::remove_file(&old.state)?;
            let _ = std::fs::remove_file(&old.thumb);
        }
        Ok(())
    }

    pub fn slots(&self) -> [Option<StateEntry>; SLOT_COUNT] {
        let mut slots: [Option<StateEntry>; SLOT_COUNT] = Default::default();
        let Ok(dir) = std::fs::read_dir(&self.dir) else {
            return slots;
        };
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some(STATE_EXT) {
                continue;
            }
            let Some((slot, stamp)) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(slot_name)
            else {
                continue;
            };
            let newer = slots[slot].as_ref().is_none_or(|e| e.stamp < stamp);
            if newer {
                let name = slot_file(slot, &stamp);
                slots[slot] = Some(StateEntry {
                    state: self.path(&name, STATE_EXT),
                    thumb: self.path(&name, THUMB_EXT),
                    stamp,
                });
            }
        }
        slots
    }

    pub fn write_slot(
        &self,
        slot: usize,
        state: &[u8],
        thumb_png: &[u8],
        stamp: &str,
    ) -> std::io::Result<()> {
        let name = slot_file(slot, stamp);
        std::fs::create_dir_all(&self.dir)?;
        atomic_write(&self.path(&name, THUMB_EXT), thumb_png)?;
        atomic_write(&self.path(&name, STATE_EXT), state)?;
        self.clear_slot_except(slot, Some(stamp))
    }

    pub fn read_slot(&self, slot: usize) -> Option<(String, Vec<u8>, Vec<u8>)> {
        let entry = self.slots().into_iter().nth(slot).flatten()?;
        let state = std::fs::read(&entry.state).ok()?;
        let thumb = std::fs::read(&entry.thumb).unwrap_or_default();
        Some((entry.stamp, state, thumb))
    }

    pub fn clear_slot(&self, slot: usize) -> std::io::Result<()> {
        self.clear_slot_except(slot, None)
    }

    fn clear_slot_except(&self, slot: usize, keep: Option<&str>) -> std::io::Result<()> {
        let dir = match std::fs::read_dir(&self.dir) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };
        for entry in dir.flatten() {
            let path = entry.path();
            let Some((at, stamp)) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(slot_name)
            else {
                continue;
            };
            if at == slot && keep != Some(stamp.as_str()) {
                std::fs::remove_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn active_slot(&self) -> usize {
        std::fs::read_to_string(self.dir.join(ACTIVE_SLOT))
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .and_then(|n| n.checked_sub(1))
            .filter(|&n| n < SLOT_COUNT)
            .unwrap_or(0)
    }

    pub fn set_active_slot(&self, slot: usize) -> std::io::Result<()> {
        if slot >= SLOT_COUNT || slot == self.active_slot() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.dir)?;
        atomic_write(
            &self.dir.join(ACTIVE_SLOT),
            format!("{}\n", slot + 1).as_bytes(),
        )
    }

    fn path(&self, stem: &str, ext: &str) -> PathBuf {
        self.dir.join(format!("{stem}.{ext}"))
    }

    fn stamped(&self, stamp: &str, ext: &str) -> std::io::Result<PathBuf> {
        if !is_stamp(stamp) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("not a state stamp: {stamp}"),
            ));
        }
        Ok(self.path(stamp, ext))
    }
}

fn slot_file(slot: usize, stamp: &str) -> String {
    format!("slot{}-{stamp}", slot + 1)
}

fn slot_name(name: &str) -> Option<(usize, String)> {
    let rest = name.strip_prefix("slot")?;
    let (n, stamp) = rest.split_once('-')?;
    let slot = n.parse::<usize>().ok()?.checked_sub(1)?;
    (slot < SLOT_COUNT && is_stamp(stamp)).then(|| (slot, stamp.to_string()))
}

fn is_stamp(s: &str) -> bool {
    const SHAPE: &[u8] = b"0000-00-00_00-00-00";
    s.len() == SHAPE.len()
        && s.bytes().zip(SHAPE).all(|(c, shape)| match shape {
            b'0' => c.is_ascii_digit(),
            _ => c == *shape,
        })
}
