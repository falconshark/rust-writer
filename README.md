# Rust Writer 🦀✍️

A fullscreen, distraction-free writing application inspired by FocusWriter — rewritten in **Rust**.

It's a vibe-coded toy, so there may be bugs. Issues and pull requests are welcome.

## Features

- Fullscreen writing mode (F11)
- Paper-inside-viewport layout — the paper is fixed to the window height; text scrolls inside it
- Configurable text column width
- Background image (5 modes: Zoomed, Scaled, Stretched, Centered, Tiled)
- 8 built-in color themes + custom theme editor (background color, paper color/opacity, text color)
- Typewriter sounds — synthesised key-click on every keystroke (no audio files needed)
- Multi-document tabs
- Auto-save
- Word / character count
- Daily word goal with progress bar
- Typewriter mode (cursor centering)
- CJK input method support (IME)
- Native file dialogs
- Persistent settings (TOML)
- Keyboard shortcuts

## Building

### Prerequisites

Rust toolchain ≥ 1.75:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Linux system dependencies:
```bash
sudo apt install \
  libgtk-3-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  libasound2-dev
```

macOS / Windows: no extra deps needed.

### Run

```bash
cargo run --release
```

### Hot reload during development

```bash
cargo install cargo-watch
cargo watch -x run
```

## Architecture

```
rust_writer/
├── Cargo.toml
├── assets/
│   ├── icon.png
│   └── fonts/
│       └── NotoSansTC-Regular.ttf
└── src/
    ├── main.rs        # Entry point, font/window/icon setup
    ├── app.rs         # App state + egui update loop + rendering
    ├── document.rs    # Document + multi-tab DocumentManager
    ├── theme.rs       # 8 built-in color themes
    ├── settings.rs    # Persistent TOML settings + BgImageMode
    ├── sounds.rs      # Synthesised typewriter click audio
    ├── toolbar.rs     # Toolbar state
    └── word_count.rs  # Word / char counting utilities
```

## Keyboard Shortcuts

| Shortcut        | Action                  |
|-----------------|-------------------------|
| Ctrl+N          | New document            |
| Ctrl+O          | Open file               |
| Ctrl+S          | Save                    |
| Ctrl+Shift+S    | Save As                 |
| F11             | Toggle fullscreen       |
| Escape          | Exit fullscreen         |
| F5              | Toggle typewriter mode  |
| Ctrl+Tab        | Next tab                |
| Ctrl+,          | Settings                |
| Ctrl+Home       | Scroll to top           |
| Ctrl+End        | Scroll to bottom        |

## Themes

Night Owl · Solarized Dark · Solarized Light · Typewriter · Forest · Midnight Blue · Paper White · Dracula

## License

GPL-3.0
