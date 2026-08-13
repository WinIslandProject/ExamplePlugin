# WinIsland Example

An open-source reference plugin for the WinIsland marketplace. It registers a
2×2 widget and renders through WinIsland's native Draw API, so the plugin does
not depend on Skia.

## Features

- Native ABI v1 lifecycle
- Widget capability only
- Host-provided text, shape, and progress-ring drawing
- Clean shutdown and resource release

## Build

```powershell
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --example pack
```

The source is intentionally small enough to use as a starting point for a new
plugin.
