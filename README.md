# dtxpt

DTXMania-like drum rhythm game in Rust and [Bevy](https://bevyengine.org/) 0.18.
Behavior target is official DTXMania 4.x — not a port of DTXManiaNX.

## Quick start

```bash
cargo check
cargo test
cargo run                          # fullscreen borderless (default)
DTXPT_WINDOWED=1 cargo run         # 1280×780 window
cargo run -- path/to/chart.dtx     # dev shortcut: skip to that chart path
```

Put charts under `charts/` (gitignored) or set **Chart root** in Settings (`Tab` → General).

## Controls (summary)

| Context | Keys |
|---------|------|
| Main menu | `Enter` song select |
| Song select | arrows / `PgUp` `PgDn`, `Enter` play, `/` search, `Esc` back |
| Settings overlay | `F1` toggle, `Tab` category, `/` search, arrows adjust |
| Gameplay | lane keys `A S D F G H J K L ;`, `Space` pause, `R` restart |

## License

TBD.
