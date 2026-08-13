# WinIsland Example

A minimal open-source WinIsland widget. It renders `Plugin loaded` with a line
under the text through WinIsland's native Draw API.

```powershell
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --example pack
```
