# Passbuddy — Hardware KeePass for ESP32-S3

Passbuddy is a no_std Rust firmware for the ESP32-S3 that acts like a USB keyboard (HID) to auto-type passwords stored in an on-device KeePass database. The database lives in flash and is encrypted with a software key derived from a user PIN plus the chip’s efuse keyslot 0 (unreadable externally), so secrets never leave the device.

## How It Works
- **Key derivation:** User PIN + IFUSE/HMAC using keyslot 0 → one-time KDF output (software key). The hardware root key never leaves the chip, and the PIN never leaves the device. The software key encrypts/decrypts the KeePass DB.
- **Usage model:** Plug the device; it enumerates as a USB keyboard. Enter PIN locally (buttons/rotary/etc). The DB decrypts on-device. When a login field is focused, the device types the selected password over USB keystrokes.
- **Security posture:** No cloud or sync. Flash theft is useless without the PIN and bypassing S3 anti-dump protections. Vendor compromise is irrelevant because secrets stay on hardware.

## Project Layout
- `src/bin/main.rs` — ESP-RTOS entrypoint; sets CPU clocks, HMAC peripheral, SPI2, and ST7789 display; draws a simple `ratatui` list via `mousefood`; logs a heartbeat.
- `src/display.rs` — display helpers (`init_terminal`, `initial_state`, `draw_menu`).
- `src/encryption.rs` — HMAC-based software key derivation (`derive_sw_key`).
- `build.rs` — adds linker scripts (`defmt.x`, `linkall.x`) and prints hints for missing symbols.
- `.cargo/config.toml` — targets `xtensa-esp32s3-none-elf`, sets `espflash` runner, enables `build-std` for `core`/`alloc`.

## Prereqs
- Rust toolchain channel `esp` (see `rust-toolchain.toml`); install Xtensa support via Espressif’s toolchain (e.g., `espup`).
- `espflash` installed and USB permissions set (dialout/uaccess on Linux).

## Build, Flash, and Dev
```bash
cargo build                 # dev build for ESP32-S3
cargo build --release       # size-optimized
cargo clippy --no-deps      # lint; keep warnings at zero
cargo run                   # flash + defmt monitor via espflash (device attached)
```
Notes: ensure only one serial/monitor session is open when flashing; replug USB if flashing stalls.

## Architecture Notes
- No_std + Embassy executor; heap set via `esp_alloc::heap_allocator!` in `main`.
- Display: ST7789 over SPI2 using `mipidsi`; UI rendered with `ratatui`/`mousefood`.
- HID keyboard output planned for password typing; input hardware for PIN entry is TBD.

## Status / Next Steps
- ✅ Display init and static menu render.
- ✅ Software key derivation helper using HMAC + RNG.
- ⏳ Implement PIN input, KeePass DB storage/crypto flow, HID keyboard typing, and runtime UI updates (via Embassy task).
