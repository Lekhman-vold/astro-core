# astro-core

Minimal Rust wrapper around the Swiss Ephemeris C library for computing Sun, Moon, and Ascendant signs from UTC birth data and location.

## Features
- Safe Rust API (`calculate_core_chart`) returning Sun, Moon, and Ascendant signs.
- Uses vendored Swiss Ephemeris C sources compiled via `cc` in `build.rs`.
- Configurable ephemeris data path (`set_ephe_path`); defaults to current dir.

## Getting Started
```bash
cargo build
cargo test            # runs unit tests
cargo run --example basic
```

## Usage
```rust
use astro_core::{calculate_core_chart, set_ephe_path, BirthData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_ephe_path("src/swisseph/ephe"); // adjust if your ephemeris files live elsewhere

    let birth = BirthData {
        year: 1990,
        month: 7,
        day: 15,
        hour: 10,
        minute: 30,
        second: 0.0,
        lat: 40.7128,
        lon: -74.0060,
    };

    let chart = calculate_core_chart(&birth)?;
    println!("Sun: {}, Moon: {}, Asc: {}", chart.sun_sign, chart.moon_sign, chart.asc_sign);
    Ok(())
}
```

## Ephemeris Data
- The Swiss Ephemeris data and C sources are vendored under `src/swisseph`.
- If you use external ephemeris files, point to them with `set_ephe_path`.
- Swiss Ephemeris is licensed separately; review `src/swisseph/LICENSE`.

## Development
- Formatting/lint: `cargo fmt`, `cargo clippy -- -D warnings`.
- Example run: `cargo run --example basic`.
- Warnings from the Swiss Ephemeris C code are upstream and expected; silence with `.warnings(false)` in `build.rs` if needed.

## Author
Volodymyr Lekhman — [LinkedIn](https://www.linkedin.com/in/volodymyr-lekhman-0a40121ab/)
