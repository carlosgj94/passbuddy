# Repository Guidelines

## Project Structure & Module Organization
- `src/bin/main.rs`: no_std entrypoint under ESP-RTOS; brings up clocks, SPI2, the ST7789 display (`mipidsi`), and draws a small `ratatui` list via `mousefood`.
- `src/encryption.rs`: HMAC-based key derivation using `esp-hal` RNG/HMAC; keep cryptographic helpers here.
- `src/display.rs`: reserved for reusable display/widgets as UI grows.
- `build.rs`: injects linker scripts (`defmt.x`, `linkall.x`) and prints hints for missing symbols.
- `.cargo/config.toml`: sets target `xtensa-esp32s3-none-elf`, uses `espflash` runner, and enables `build-std` for `core`/`alloc`.

## Build, Flash, and Development Commands
- `cargo build` — builds for ESP32-S3 with the `esp` toolchain from `rust-toolchain.toml`.
- `cargo run` — flashes and opens a defmt monitor via `espflash flash --monitor --chip esp32s3`; ensure only one monitor session is active.
- `cargo build --release` — optimized binary for on-device checks.
- `cargo clippy --no-deps` — lint; keep warnings at zero.
- If flashing fails, replug USB and confirm permissions (uaccess/dialout).

## Coding Style & Naming Conventions
- Rust 2024, 4-space indent, `rustfmt` default; keep `#![no_std]` modules free of `std`.
- Log with `defmt::{info, warn, error}`; avoid `println!`.
- Use `snake_case` for items, `UpperCamelCase` for types; organize modules by hardware domain (`display`, `encryption`, input).

## Testing Guidelines
- No automated tests yet; add unit tests to pure helpers behind `#[cfg(test)]` and run on host with `cargo test --target x86_64-unknown-linux-gnu` (install target first).
- For device validation, flash with `cargo run` and watch defmt logs; keep log noise low for serial bandwidth.
- When adding peripherals, include a minimal init check and log failures.

## Commit & Pull Request Guidelines
- Follow conventional commits seen here (`feat: ...`); explain scope and motivation.
- Keep PRs focused; include behavior summary, risk notes, and hardware/test evidence (board, command run, key logs or screenshots).
- Link related issues when available.

## Security & Configuration Tips
- Do not commit secrets, Wi-Fi credentials, or device-specific keys; use local env/flash storage.
- The heap is set in `main` via `esp_alloc::heap_allocator!`; ensure new allocations fit within the reclaimed RAM budget.
- Reuse HMAC key slots intentionally; document changes to cryptographic material.
