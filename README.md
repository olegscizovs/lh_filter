# LHFilter

**LHFilter** is a simple VST3 audio plugin that features highly optimized Low-Pass and High-Pass filters, alongside precise Q-Factor resonance control. Built on the blazing-fast Rust `nih-plug` framework and featuring a custom `egui` graphical interface, it is designed for both performance and premium user experience.

## Features
- **Low-Pass & High-Pass Filters**: 20 Hz to 20,000 Hz coverage utilizing flawless mathematical logarithmic scaling (`FloatRange::Skewed`) for smooth, natural frequency sweeping.
- **Q-Factor Control**: Linear control over filter resonance (0.1 to 10.0).
- **Custom UI System**: Hand-coded rotary knob widgets that bypass generic slider limitations.
- **Precision Tuning**: Hold `Ctrl` (or `Cmd` on Mac) while dragging or scrolling the knobs to enter precision mode (10x slower movement), accompanied by a distinct blue visual indicator.
- **Bidirectional Sync**: Type exact values into the text fields to immediately update the knobs, or rotate the knobs to watch the text fields update in real time. 
- **Quick Reset**: Double-click any knob to instantly snap it back to its optimal default value.

## Technology Stack
- **Language**: Rust
- **Plugin Framework**: `nih-plug`
- **GUI Framework**: `nih_plug_egui`
- **Build System**: `cargo xtask`

## Building from Source

This repository utilizes the `xtask` pattern to bundle the plugin directly into a `.vst3` directory format.

### Requirements
- Rust (latest stable)
- `cargo` 
- A supported cross-compilation toolchain if building for other platforms (e.g., `x86_64-w64-mingw32-gcc` for Windows cross-compilation from Linux).

### Compilation
To build the plugin and generate the bundled VST3 file, run:

```bash
cargo xtask bundle lh_filter --release
```

#### Windows Cross-Compilation (From Linux)
If you are on Linux and wish to compile the Windows `.vst3` plugin, use the following target flag:
```bash
cargo xtask bundle lh_filter --release --target x86_64-pc-windows-gnu
```

The resulting `lh_filter.vst3` bundle will be located in the `target/bundled/` directory.

## Installation
Copy the generated `.vst3` directory into your DAW's designated VST3 folder:
- **Windows**: `C:\Program Files\Common Files\VST3\`
- **Linux**: `~/.vst3/` or `/usr/lib/vst3/`
- **macOS**: `/Library/Audio/Plug-Ins/VST3/`

## License
MIT License.
