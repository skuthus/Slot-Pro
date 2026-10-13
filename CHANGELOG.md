# Changelog

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
