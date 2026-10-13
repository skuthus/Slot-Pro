<div align="center">

<img src=".github/resources/slot-pro-logo.png" width="100%" alt="slot pro.">

</div>

Slot Pro is a Game Boy-centric frontend for the Anbernic RG SP and other H700 handhelds. It plays
GBA, GB and GBC games from a cartridge carousel, and builds on [slot](https://github.com/BrandonKowalski/slot)
by Brandon T. Kowalski.

<p align="center">
<img src=".github/resources/screenshots/carousel.gif" width="48%" alt="The cartridge carousel across the GBA, GB and GBC shelves">
<img src=".github/resources/screenshots/core-picker.gif" width="48%" alt="Choosing between mGBA and gpSP for a GBA cart">
</p>

## Features

- **Favorites**: star games with Y and find them on their own shelf, optionally opening there.
- **Cartridge carousel** with label art, per-platform shelves, an optional game title, and a 12 or
  24 hour clock.
- **Three save slots per game**, each with a screenshot. Open them with SELECT on the carousel or
  from the in-game menu.
- **In-game menu** (tap MENU): resume, save and load states, shaders, controls and eject.
- **Button remapping**, globally or per game, for A, B, L, R, SELECT, START, X and Y.
- **Analog Mode**, which plays like a real GBA: no save states, menus, rewind, fast forward, turbo
  or resume. Only the game's own save survives power off.
- **GBC Frame**, which surrounds GB and GBC games with a Game Boy Color shell in six colors.
- **Display options**: screen shaders including Authentic GBC (GBC colors, pixel grid, see-through
  pixels and LCD ghosting), Game Boy palettes, and color correction for GBA only, GB and GBC only,
  all games, or none.
- **Power button**: tap for Sleep, Restart or Power Off. Hold to save and shut down.
- **Fast boot**: compiled shaders are cached on the card, and a resumed game loads while the screen
  starts.
- **mGBA, gpSP and Gambatte cores**, chosen per game, plus rewind, run-ahead, fast forward, turbo,
  rumble, and link play between two devices.

## Supported devices

| Device             | Notes                     |
|--------------------|---------------------------|
| Anbernic RG SP     | Primary target            |
| Anbernic RG34XX    |                           |
| Anbernic RG34XXSP  | Analog stick not used     |
| Anbernic RG35XXSP  | GBA SP bezel below the screen |
| Anbernic RG35XX Plus |                         |
| Anbernic RG28XX    | Portrait panel. If the picture is upside down, add `export SLOT_ROTATE=ccw` to `System/launch_frontend.sh` |

## Installation

Slot Pro runs on [BaseOS](https://github.com/pvaibhav/BaseOS).

1. Write the latest BaseOS image to an SD card, insert it into SD1, and power on once so it expands
   its storage. Power off when it reports that no frontend was found.
2. Copy the contents of a Slot Pro build onto the second card (SD2), formatted FAT32 or exFAT with an
   MBR partition table. A single card in SD1 also works.
3. Add unzipped ROMs to `Games/GBA`, `Games/GB` and `Games/GBC`. `.zip` and `.7z` files are not read.
4. Optionally add label art as `Labels/<system>/<rom name>.png`, for example
   `Labels/GBA/Metroid Fusion.png` for `Games/GBA/Metroid Fusion.gba`.

To update, replace the `System/` folder on the card. Games, saves, states, labels and settings live
outside it.

## Controls

| Where       | Input                     | Action                                              |
|-------------|---------------------------|-----------------------------------------------------|
| Everywhere  | Tap POWER                 | Sleep, Restart or Power Off                         |
|             | Hold POWER                | Save and shut down                                  |
|             | SELECT + Up / Down        | Brightness                                          |
|             | SELECT + Left / Right     | Blue light filter                                   |
|             | VOL+ / VOL-               | Volume                                              |
|             | SELECT + VOL+ / VOL-      | Mute                                                |
| Carousel    | Left / Right, Up / Down   | Browse, or skip to the next letter                  |
|             | L1 / R1                   | Switch platform                                     |
|             | A / Hold A                | Play from where you left off / start fresh          |
|             | Y                         | Add to or remove from Favorites                     |
|             | SELECT                    | Save slots for the highlighted game                 |
|             | START                     | Choose the core: mGBA or gpSP (GBA), mGBA or Gambatte (GB, GBC) |
|             | MENU                      | Settings                                            |
| In game     | Tap MENU                  | In-game menu                                        |
|             | Double tap MENU           | Load a save slot                                    |
|             | Hold MENU                 | Eject to the carousel                               |
|             | SELECT + R1 / L1          | Save to / load from the active slot                 |
|             | X (after a save or load)  | Undo, within 30 seconds                             |
|             | SELECT + B / A            | Previous / next shader                              |
|             | SELECT + Y                | Next color correction mode                          |
|             | SELECT + L2 / R2          | Previous / next Game Boy palette                    |
|             | SELECT + MENU             | Link with another device                            |
|             | Hold L2 / Hold R2         | Rewind / fast forward (double tap R2 to lock)       |
|             | Hold X / Y                | Turbo A / B                                         |
|             | L1 / R1 (GB and GBC)      | Fill the screen / original size                     |

In Analog Mode, save, load, menu, rewind, fast forward and turbo inputs are disabled, and SELECT + L1 / R1
reach the game.

## Building

Requires Rust (the version is pinned in `rust-toolchain.toml`), [Task](https://taskfile.dev), CMake and,
for device builds, Docker.

```bash
task test          # run the test suite
task run           # run on the desktop against ./sdcard
task dist:device   # build the SD card tree for H700 devices into ./dist-device
```

## Credits

- [slot](https://github.com/BrandonKowalski/slot) by Brandon T. Kowalski, which Slot Pro is built on.
- RG28XX support by [GamerAnim](https://github.com/GamerAnim).
- Emulation by [mGBA](https://mgba.io) (MPL-2.0), [gpSP](https://github.com/libretro/gpsp) (GPL-2.0) and
  [Gambatte](https://github.com/libretro/gambatte-libretro) (GPL-2.0).
- [BaseOS](https://github.com/pvaibhav/BaseOS) by pvaibhav.
- Open Sans, Nerd Fonts symbols, and Cascadia Code (SIL Open Font License) for the boot logo.

Game Boy, Game Boy Color and Game Boy Advance are trademarks of Nintendo. Slot Pro is not affiliated
with Nintendo or Anbernic.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
