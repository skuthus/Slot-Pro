use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::Platform;

const NAMES: [&str; 65] = [
    "Grayscale",
    "DMG Green",
    "GB Pocket",
    "GB Light",
    "GBC Brown ↑",
    "GBC Red ↑A",
    "GBC Dark Brown ↑B",
    "GBC Pale Yellow ↓",
    "GBC Orange ↓A",
    "GBC Yellow ↓B",
    "GBC Blue ←",
    "GBC Dark Blue ←A",
    "GBC Gray ←B",
    "GBC Green →",
    "GBC Dark Green →A",
    "GBC Reverse →B",
    "SGB 1-A",
    "SGB 1-B",
    "SGB 1-C",
    "SGB 1-D",
    "SGB 1-E",
    "SGB 1-F",
    "SGB 1-G",
    "SGB 1-H",
    "SGB 2-A",
    "SGB 2-B",
    "SGB 2-C",
    "SGB 2-D",
    "SGB 2-E",
    "SGB 2-F",
    "SGB 2-G",
    "SGB 2-H",
    "SGB 3-A",
    "SGB 3-B",
    "SGB 3-C",
    "SGB 3-D",
    "SGB 3-E",
    "SGB 3-F",
    "SGB 3-G",
    "SGB 3-H",
    "SGB 4-A",
    "SGB 4-B",
    "SGB 4-C",
    "SGB 4-D",
    "SGB 4-E",
    "SGB 4-F",
    "SGB 4-G",
    "SGB 4-H",
    // Community favourites from Lospec, which slot patches into both cores under these names.
    "Ice Cream GB",
    "Kirokaze",
    "2bit Demichrome",
    "Hollow",
    "Mist GB",
    "Rustic GB",
    "AYY4",
    "BGB Green",
    "Wish GB",
    "Nostalgia",
    "Crimson",
    "BLK AQU4",
    "SpaceHaze",
    "Velvet Cherry GB",
    "Gold GB",
    "Pokemon SGB",
    "Link's Awakening SGB",
];

const COMMUNITY_FROM: usize = 48;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GbPalette(u8);

impl GbPalette {
    pub const COUNT: usize = NAMES.len();
    pub const DEFAULT: GbPalette = GbPalette(1);

    pub fn all() -> impl Iterator<Item = GbPalette> {
        (0..Self::COUNT as u8).map(GbPalette)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn core_name(self) -> &'static str {
        NAMES[self.index()]
    }

    /// The name Gambatte's `gambatte_gb_internal_palette` gives the same colours.
    pub fn gambatte_name(self) -> String {
        let name = self.core_name();
        if self.community() {
            return name.to_string();
        }
        if let Some(sgb) = name.strip_prefix("SGB ") {
            return format!("SGB - {}", sgb.replace('-', ""));
        }
        match self.label() {
            "Grayscale" | "GBC Gray" => "GBC - Grayscale",
            "DMG Green" => "GB - DMG",
            "GB Pocket" => "GB - Pocket",
            "GB Light" => "GB - Light",
            "GBC Brown" => "GBC - Brown",
            "GBC Red" => "GBC - Red",
            "GBC Dark Brown" => "GBC - Dark Brown",
            "GBC Pale Yellow" => "GBC - Pastel Mix",
            "GBC Orange" => "GBC - Orange",
            "GBC Yellow" => "GBC - Yellow",
            "GBC Blue" => "GBC - Blue",
            "GBC Dark Blue" => "GBC - Dark Blue",
            "GBC Green" => "GBC - Green",
            "GBC Dark Green" => "GBC - Dark Green",
            "GBC Reverse" => "GBC - Inverted",
            _ => "GB - DMG",
        }
        .to_string()
    }

    /// One of the Lospec favourites slot adds to the cores, rather than a console's own palette.
    pub fn community(self) -> bool {
        self.index() >= COMMUNITY_FROM
    }

    /// The order SELECT + L2 / R2 offers them: the community favourites, then the consoles' own.
    pub fn in_cycle_order() -> impl Iterator<Item = GbPalette> {
        (COMMUNITY_FROM..Self::COUNT)
            .chain(0..COMMUNITY_FROM)
            .map(|i| GbPalette(i as u8))
    }

    pub fn label(self) -> &'static str {
        let name = self.core_name();
        match name.find(['↑', '↓', '←', '→']) {
            Some(at) => name[..at].trim_end(),
            None => name,
        }
    }

    pub fn next(self) -> GbPalette {
        GbPalette(((self.index() + 1) % Self::COUNT) as u8)
    }

    pub fn prev(self) -> GbPalette {
        GbPalette(((self.index() + Self::COUNT - 1) % Self::COUNT) as u8)
    }

    pub fn parse(name: &str) -> Option<GbPalette> {
        Self::all().find(|p| p.core_name() == name)
    }
}

/// Palettes picked for Game Boy carts that neither the Game Boy Color nor the Super Game Boy
/// colours, keyed by the title in the cart header. Auto falls back to the core's own colours.
const CURATED: [(&str, &str); 21] = [
    ("AVENGING SPIRIT", "Hollow"),
    ("BUBBLE BOBBLE", "Ice Cream GB"),
    ("BUBBLE GHOST", "Wish GB"),
    ("CATRAP", "AYY4"),
    ("CAVE NOIRE", "Rustic GB"),
    ("ECLIPSE", "SGB 4-D"),
    ("GARGOYLE'S", "Crimson"),
    ("GREAT GREED", "SGB 2-A"),
    ("KID DRACULA", "SpaceHaze"),
    ("KWIRK", "Pokemon SGB"),
    ("MERCENARY FORCE", "2bit Demichrome"),
    ("NEMESIS", "BLK AQU4"),
    ("PARODIUS", "GBC Pale Yellow ↓"),
    ("POPULOUS", "GBC Brown ↑"),
    ("SAGA", "Link's Awakening SGB"),
    ("SAGA2", "Kirokaze"),
    ("SAGA3", "Mist GB"),
    ("STARGATE", "Gold GB"),
    ("SWORD OF HOPE 2", "SGB 3-E"),
    ("THE SWORD OFHOPE", "SGB 1-E"),
    ("TRIP WORLD", "SGB 3-A"),
];

/// The palette slot picked for a cart with this header title, if any.
pub fn curated_palette(title: &str) -> Option<GbPalette> {
    CURATED
        .iter()
        .find(|(t, _)| *t == title)
        .and_then(|(_, name)| GbPalette::parse(name))
}

/// The title in a Game Boy cart header, as the GBC and Super Game Boy look it up.
pub fn gb_header_title(header: &[u8]) -> String {
    header
        .get(0x134..0x144)
        .unwrap_or_default()
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect::<String>()
        .trim_end()
        .to_string()
}

fn game_palette_path(root: &Path, platform: Platform, stem: &str) -> PathBuf {
    root.join("Config")
        .join("Palettes")
        .join(platform.dir_name())
        .join(format!("{stem}.txt"))
}

/// The palette chosen for one game with SELECT + L2 / R2. None leaves it on Auto.
pub fn read_game_palette(root: &Path, platform: Platform, stem: &str) -> Option<GbPalette> {
    let text = std::fs::read_to_string(game_palette_path(root, platform, stem)).ok()?;
    GbPalette::parse(text.trim())
}

pub fn write_game_palette(
    root: &Path,
    platform: Platform,
    stem: &str,
    palette: Option<GbPalette>,
) -> std::io::Result<()> {
    let path = game_palette_path(root, platform, stem);
    match palette {
        Some(p) => {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            atomic_write(&path, format!("{}\n", p.core_name()).as_bytes())
        }
        None => match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curated_title_names_a_palette_both_cores_carry() {
        for (title, name) in CURATED {
            assert!(
                GbPalette::parse(name).is_some(),
                "{title}: no palette {name:?}"
            );
        }
    }
}
