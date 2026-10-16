# Changelog

## Unreleased

### Fixed

- Rumble works in Gambatte. It used to read its rumble level as 0 because slot never set it.

## 1.1.2 (2026-10-16)

### Added

- Three darker brightness steps below the old lowest. Saved brightness moves up to keep the same light.

### Fixed

- Gambatte with Run-Ahead no longer buzzes or sounds rough, and LCD Ghosting keeps blending with it on. Run-Ahead's rollbacks used to reset Gambatte's audio resampler every frame.
- Saves and states reach the card before slot carries on, so a power cut right after a save can no longer leave the card read-only with the save's file entry pointing at freed space.

## 1.1.1 (2026-10-15)

### Changed

- Gambatte is the default core for GB and GBC games. Games already set to mGBA with START keep it; others start from their in-game save on Gambatte, and their mGBA states return if switched back.
- The in-game menu shows the battery, its percentage and the clock.

## 1.1.0 (2026-10-13)

### Added

- Favorites: press Y on the carousel to star a game. Starred games from every console appear A to Z on a Favorites shelf, first in the L1 / R1 cycle.
- Start on Favorites (Gameplay): open the carousel on Favorites when there is no game to resume.
- Gambatte core for GB and GBC games, chosen per game with START on the carousel. Palettes and color correction carry over, and cartridge clocks survive switching cores.
- Run-Ahead (Gameplay): Off, 1 or 2 frames, to cut input lag at the cost of more CPU.
- Authentic GBC shader for GB and GBC games: GBC screen colors, a soft pixel grid and see-through pixels. With it on, the core's own color correction steps aside so colors aren't corrected twice.
- LCD Ghosting (Screen): with Authentic GBC, frames blend like the GBC's slow LCD, so games that flicker sprites for transparency look see-through. On by default.

## 1.0.0 (2026-10-13)

Based on slot 1.6.0. For earlier history, see the
[slot changelog](https://github.com/BrandonKowalski/slot/blob/main/CHANGELOG.md).

### Added

- Three save slots per game, each with a screenshot and save time. They replace the ten-state history.
- SELECT on the carousel opens the highlighted game's slots, showing the slot's screenshot in place of the cart. A starts the game from that slot.
- In-game menu (tap MENU): Resume, Save State, Load State, Shader, Controls and Eject Game. Double tap MENU opens Load State directly.
- Button remapping for all games (Controls on the carousel menu) or per game (Controls in the in-game menu). A, B, L, R, SELECT, START, X and Y can each send any GBA button; X and Y keep turbo by default.
- Analog Mode (Gameplay): no save states, in-game menu, rewind, fast forward, turbo or resume. Only the game's own save survives power off, and SELECT + L1 / R1 reach the game.
- GBC Frame (Screen): GB and GBC games play inside a Game Boy Color shell in Berry, Grape, Kiwi, Dandelion, Teal or Atomic Purple.
- Game Title (Screen): shows the selected game's name below or above its cart.
- Clock (Screen): 12 or 24 hour time.

### Changed

- Tap POWER for Sleep, Restart or Power Off; hold POWER to save and shut down. Closing the lid still sleeps.
- Color Correction has four modes: Off, GBA Only, GB / GBC and All. SELECT + Y steps through them.
- SELECT + R1 / L1 save to and load from the active slot, and the message names the slot.
- Faster boot: compiled shaders are cached in `System/Cache/shaders`, and a resumed game's files are read while the screen starts.
- New "slot pro." boot logo and About label.
