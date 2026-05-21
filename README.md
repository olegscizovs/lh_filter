# lh_filter V1

**lh_filter V1** is a simple VST3 audio plugin featuring Low-Pass and High-Pass filters with Q-Factor resonance control and a built-in soft limiter. Built with Rust using the `nih-plug` framework and a custom `egui` graphical interface.

## Features
- **Low-Pass & High-Pass Filters**: 20 Hz to 20,000 Hz coverage with logarithmic scaling (`FloatRange::Skewed`) for smooth, natural frequency sweeping.
- **Q-Factor Control**: Linear control over filter resonance (0.1 to 10.0).
- **Soft Limiter**: Built-in `tanh` saturation stage prevents clipping when resonance peaks from combined LP/HP filters exceed 0 dBFS. Engages automatically — clean output at all settings.
- **Custom UI**: Hand-coded rotary knob widgets with a subtle black-to-grey gradient background.
- **Precision Tuning**: Hold `Shift` while dragging or scrolling knobs for 10x finer control, with a blue visual indicator.
- **Bidirectional Sync**: Type exact values into text fields to update knobs, or rotate knobs to update text fields in real time.
- **Quick Reset**: Double-click any knob to snap back to its default value.
- **About Window**: Footer button opens an overlay with plugin info and contact details.
- **Bypass**: Global bypass switch to pass audio through unprocessed.
- **Soft Limiter**: Built-in `tanh` saturation stage prevents clipping when resonance peaks from combined LP/HP filters exceed 0 dBFS. Engages automatically — clean output at all settings.
- **Custom UI**: Hand-coded rotary knob widgets with a subtle black-to-grey gradient background.
- **Precision Tuning**: Hold `Shift` while dragging or scrolling knobs for 10x finer control, with a blue visual indicator.
- **Bidirectional Sync**: Type exact values into text fields to update knobs, or rotate knobs to update text fields in real time.
- **Quick Reset**: Double-click any knob to snap back to its default value.
- **About Window**: Footer button opens an overlay with plugin info and contact details.
- **Bypass**: Global bypass switch to pass audio through unprocessed.

## Technology Stack
- **Language**: Rust
- **Plugin Framework**: `nih-plug`
- **GUI Framework**: `nih_plug_egui`
- **Build System**: `cargo xtask`

## Building from Source

This repository uses the `xtask` pattern to bundle the plugin into a `.vst3` directory format.

### Requirements
- Rust (latest stable)
- `cargo`
- `cargo`
- A supported cross-compilation toolchain if building for other platforms (e.g., `x86_64-w64-mingw32-gcc` for Windows cross-compilation from Linux).

### Compilation
To build the plugin and generate the bundled VST3 file:

```bash
cargo xtask bundle lh_filter --release --frozen
```

#### Windows Cross-Compilation (From Linux)
```bash
cargo xtask bundle lh_filter --release --target x86_64-pc-windows-gnu --frozen
```

The resulting `lh_filter.vst3` bundle will be located in the `target/bundled/` directory.

## Installation
Copy the generated `.vst3` directory into your DAW's designated VST3 folder:
- **Windows**: `C:\Program Files\Common Files\VST3\`
- **Linux**: `~/.vst3/` or `/usr/lib/vst3/`
- **macOS**: `/Library/Audio/Plug-Ins/VST3/`

The plugin will appear in your DAW under **Creator → lh_filter V1**.

## Author
Created by Oleg Chizhov aka Чеширьsky

Contact: jaqueole@gmail.com

## License
MIT License.


