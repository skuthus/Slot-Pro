use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::platform::Platform;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PadKey {
    A,
    B,
    L,
    R,
    Select,
    Start,
    X,
    Y,
}

impl PadKey {
    pub const ALL: [PadKey; 8] = [
        PadKey::A,
        PadKey::B,
        PadKey::L,
        PadKey::R,
        PadKey::Select,
        PadKey::Start,
        PadKey::X,
        PadKey::Y,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            PadKey::A => "a",
            PadKey::B => "b",
            PadKey::L => "l",
            PadKey::R => "r",
            PadKey::Select => "select",
            PadKey::Start => "start",
            PadKey::X => "x",
            PadKey::Y => "y",
        }
    }

    pub fn parse(name: &str) -> Option<PadKey> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }

    pub fn step(self, right: bool) -> PadKey {
        let n = Self::ALL.len();
        let at = self.index();
        Self::ALL[if right {
            (at + 1) % n
        } else {
            (at + n - 1) % n
        }]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Layout([PadKey; 8]);

impl Default for Layout {
    fn default() -> Self {
        Layout(PadKey::ALL)
    }
}

impl Layout {
    pub fn sends(self, pressed: PadKey) -> PadKey {
        self.0[pressed.index()]
    }

    pub fn set(&mut self, pressed: PadKey, sends: PadKey) {
        self.0[pressed.index()] = sends;
    }

    pub fn name(self) -> String {
        self.0.map(PadKey::name).join(",")
    }

    pub fn parse(text: &str) -> Option<Layout> {
        let mut keys: Vec<PadKey> = text
            .trim()
            .split(',')
            .map(|k| PadKey::parse(k.trim()))
            .collect::<Option<_>>()?;
        if keys.len() == PadKey::ALL.len() - 2 {
            keys.extend([PadKey::X, PadKey::Y]);
        }
        Some(Layout(keys.try_into().ok()?))
    }
}

fn game_layout_path(root: &Path, platform: Platform, stem: &str) -> PathBuf {
    root.join("Config")
        .join("Layouts")
        .join(platform.dir_name())
        .join(format!("{stem}.txt"))
}

pub fn read_game_layout(root: &Path, platform: Platform, stem: &str) -> Option<Layout> {
    let text = std::fs::read_to_string(game_layout_path(root, platform, stem)).ok()?;
    Layout::parse(&text)
}

pub fn write_game_layout(
    root: &Path,
    platform: Platform,
    stem: &str,
    layout: Option<Layout>,
) -> std::io::Result<()> {
    let path = game_layout_path(root, platform, stem);
    match layout {
        Some(layout) => {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            atomic_write(&path, format!("{}\n", layout.name()).as_bytes())
        }
        None => match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
    }
}
