# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ff11sim is a FINAL FANTASY XI character, equipment and damage simulator. written in Rust. It calculates character stats based on race, job, and level using FFXI's actual game formulas.

1. Search the equipments database.
2. Create and register equipsets.
3. Calculation character's status by setting race, jobs, levels and equipsets.
4. Simulation damages by the character created by 3.

## Build and Test Commands

All commands should be run from the `rust/` directory:

```bash
cargo build              # Build the project
cargo build --release    # Build optimized release
cargo run                # Run the binary
cargo test               # Run all tests
cargo test <test_name>   # Run a specific test (e.g., cargo test chara_builder)
cargo fmt                # Format code
cargo clippy             # Run linter
```

## Architecture

The codebase is organized into four main modules in `rust/src/`:

### Module Structure

```
lib.rs          # Module exports
main.rs         # Entry point (minimal, ready for CLI development)
├── chara.rs    # Character struct with builder pattern
├── job.rs      # Job enum (22 FFXI jobs)
├── race.rs     # Race enum with status grade lookup table
└── status.rs   # Status calculation engine
```

### Key Concepts

**Character (`chara.rs`)**: Uses a builder pattern (`CharaBuilder`) to construct characters with:
- `race`: One of 5 races (Hum, Elv, Tar, Mit, Gal)
- `main_job` + `main_lv`: Primary job and level (1-99)
- `support_job` + `support_lv`: Optional sub-job (1-99)
- `master_lv`: Master level (0-50)

**Status Calculation (`status.rs`)**: Implements FFXI's stat formula using:
- Grade system (A-G) for stat scaling
- Piecewise calculation with different coefficients for level ranges (2-60, 61-75, 76-99)
- Special HP/MP bonus calculation after level 30
- Lookup tables: `GRADE_COEF_HPMP` and `GRADE_COEF_BP`

**Race (`race.rs`)**: Contains `STATUS_GRADES` matrix mapping each race to status grades for all 9 stats (HP, MP, STR, DEX, VIT, AGI, INT, MND, CHR).

### Dependencies

- `clap`: CLI argument parsing (derive feature)
- `enum-map`: Efficient enum-based maps
- `strum`: Enum iteration and counting utilities

## Development Notes

- Status calculation currently only uses race grades; job and master level modifiers are not yet implemented (see TODO in `chara.rs:138`)
- The `Job` enum is defined but not yet integrated into status calculations
