# FocusWriter RS 🦀✍️

A fullscreen, distraction-free writing application inspired by FocusWriter — rewritten in **Rust**.

## Features

- Fullscreen writing mode (F11)
- Configurable text column width
- Solid / gradient / image backgrounds
- 8 built-in color themes + custom theme editor
- Multi-document tabs
- Auto-save
- Word / character count
- Daily word goal with progress bar
- Typewriter mode (cursor centering)
- Focus mode (dim other paragraphs)
- Native file dialogs (rfd)
- Persistent settings (TOML)
- Keyboard shortcuts

## Building

### Prerequisites

Rust toolchain >= 1.75:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Linux additional deps:
```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                 libxkbcommon-dev libssl-dev
```

macOS / Windows: no extra deps needed.

### Run

```bash
cargo run --release
```

## Architecture

```
focuswriter-rs/
├── Cargo.toml
├── assets/
│   ├── icon.png
│   └── fonts
└── src/
    ├── main.rs        # Entry point, font/window setup
    ├── app.rs         # Main app state + egui update loop
    ├── document.rs    # Document + multi-tab DocumentManager
    ├── background.rs  # Solid / gradient / image backgrounds
    ├── theme.rs       # 8 color themes
    ├── settings.rs    # Persistent TOML settings
    ├── toolbar.rs     # Toolbar state
    └── word_count.rs  # Word/char counting utilities
```

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+N | New document |
| Ctrl+O | Open file |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |
| F11 | Toggle fullscreen |
| Escape | Exit fullscreen |
| F5 | Toggle typewriter mode |
| F6 | Toggle focus mode |
| Ctrl+Tab | Next tab |
| Ctrl+, | Settings |

## Themes

Night Owl, Solarized Dark, Solarized Light, Typewriter, Forest, Midnight Blue, Paper White, Dracula

## License

GPL-3.0
