default:
    @just --list

# Build the game (release)
[group('release')]
build-release:
    cargo build --release

# Run the game in release mode
[group('release')]
run-release:
    cargo run --release

# Build the game (debug)
[group('dev')]
build:
    cargo build  --features bevy/dynamic_linking

# Run the game with faster incremental builds
[group('dev')]
run:
    cargo run --features bevy/dynamic_linking

# Run the game with faster incremental builds
[group('dev')]
test:
    cargo test --features bevy/dynamic_linking

# Clippy: static analysis
[group('dev')]
lint:
    cargo clippy --all-targets --features bevy/dynamic_linking

# Format the code
[group('dev')]
format:
    cargo fmt

# Check formatting without changing files
[group('CI')]
format-check:
    cargo fmt --check

# Format and lint in one go
[group('dev')]
check: format lint
