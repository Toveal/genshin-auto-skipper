# Genshin Auto Skipper

Automatic dialogue skipper for Genshin Impact: a single portable `.exe` with a
small window, no Python and no installation. It is a drop-in alternative to
[genshin-dialogue-autoskip](https://github.com/1hubert/genshin-dialogue-autoskip)
with the same detection logic and timing, plus a few extras.

> [!WARNING]
> Using third-party software with Genshin Impact is against the game's Terms of
> Service and may result in a ban. Use at your own risk.

## Features

- Detects playing dialogue lines and dialogue options by sampling a few pixels
  of the game window; ignores loading screens.
- Advances lines with the interaction key (**F** by default) and picks the
  bottom option either with the confirm key or with a mouse click.
- Human-like timing: random 120–180 ms between presses (sometimes up to
  200 ms) and occasional 3–8 s breaks. All of it is configurable.
- *Just press the key* mode: presses the interaction key at the same random
  interval without looking at the screen, for cases where detection does not
  work (unusual resolution, HDR, mods).
- Works only while the game window is in focus.
- Any resolution: 16:9 scales proportionally, ultrawide (21:9, 32:9) and
  2880×1800 use dedicated positions. Windowed mode and non-primary monitors
  are supported.
- Global and Chinese (原神) clients.
- English and Russian interface; session statistics and an event log.
- Uses no CPU while paused.

## Usage

1. Download `genshin-auto-skipper.exe` from
   [Releases](https://github.com/Toveal/genshin-auto-skipper/releases).
2. In the game open **Settings > Other** and set **Auto-Play Story** to **Off**.
3. Run the program. It asks for administrator rights: the game runs elevated
   and ignores input from non-elevated programs.
4. Start the game and press **F8** (or the *Start* button).

### Hotkeys

| Key | Action |
|-----|--------|
| **F8** | Start |
| **F9** | Pause |
| **F10** | Exit |

Hotkeys work globally and can be changed in *Settings*. The reference script
uses F12 for exit, but Windows reserves F12 for debuggers, so it cannot be
registered as a global hotkey.

### Settings

Settings are saved automatically to
`%APPDATA%\genshin-auto-skipper\settings.toml`; the same folder has `app.log`.
The *Open settings folder* button opens it.

| Setting | Default | Notes |
|---------|---------|-------|
| Mode | Detect dialogues | Or *Just press the key* — no screen detection |
| Interaction key | F | Advances dialogue lines |
| Choosing options | Confirm key (F) | Or a mouse click on the bottom option |
| Human-like timing | on, 120–180 ms | Off: fixed delay equal to the minimum |
| Occasional breaks | on, 3–8 s | Checked every 30 s with a 1/25 chance |
| Color tolerance | 4 | Raise it if dialogues are not detected (HDR, filters) |

## Troubleshooting

- **Nothing happens.** Check that the status says *RUNNING*, the game line says
  *in focus*, the program runs as administrator and in-game Auto-Play Story is
  off.
- **“Game not found”.** The window is found by its title (*Genshin Impact* or
  *原神*); make sure the game is running.
- **Dialogues are not detected.** Disable HDR / color filters or raise *Color
  tolerance*. If your resolution is unusual, please open an issue with it and a
  screenshot.
- **A hotkey does not work.** Another program may use the key; the log says so.
  Pick a different one.

## Building

Requires Rust 1.85+ on Windows.

```sh
cargo build --release   # target/release/genshin-auto-skipper.exe
cargo test
```

Debug builds do not embed the administrator manifest (so that `cargo test`
works without elevation); run them from an elevated terminal to test input.

## How it works

- `engine::layout` computes the sampled positions for the current resolution;
  `engine::detector` turns the sampled colors into a dialogue state;
  `engine::humanizer` produces the timings. All three are pure and unit-tested.
- `engine::worker` runs on its own thread: it finds the game window, samples
  the screen, and sends input via `SendInput`.
- `platform` wraps the Win32 API (`windows-sys`) and registers global hotkeys.
- `ui` is an [egui](https://github.com/emilk/egui) window.

## License

MIT, see [LICENSE](LICENSE).
