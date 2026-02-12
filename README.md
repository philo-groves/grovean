# Grovean

## How to Build

Cross-compiling is supported by cargo and LLVM, allowing developers to build for non-native architectures.

#### Build with x86_64

- `kernel build --x86_64` (shorthand)
- `cargo build --target linker/x86_64-grovean.json`

#### Build with aarch64

- `kernel build --aarch64` (shorthand)
- `cargo build --target linker/aarch64-grovean.json`

## How to run

Running is executed through the `k1` crate. As a command line utility, it must be be installed on your system (`cargo install k1`).

#### Run with x86_64

- `kernel run --x86_64` (shorthand)
- `cargo run --target linker/x86_64-grovean.json`

#### Run with aarch64

- `kernel run --aarch64` (shorthand)
- `cargo run --target linker/aarch64-grovean.json`

## How to Test

Testing is conducted through the `kunit` crate, which is included as a dev dependency and configured in the lib.rs of each crate.

#### Test with x86_64

- `kernel test --x86_64` (shorthand)
- `cargo test --target linker/x86_64-grovean.json`

#### Test with aarch64

- `kernel test --aarch64` (shorthand)
- `cargo test --target linker/aarch64-grovean.json`

#### Test Specific Crate

To only test a single crate, instead of the entire workspace, pass the `-p` flag:

`cargo test -p <some-crate> --target linker/<arch>-grovean.json`

## `kernel` shorthand script (Linux/WSL)

This repository includes a root-level `kernel` script that wraps cargo commands with shorthand architecture flags.

Supported commands:

- `build`
- `run`
- `test`
- `clean`

Supported architecture flags:

- `--86_64` (alias: `--x86_64`)
- `--aarch64`

At least one architecture must be supplied for `build`, `run`, and `test`. If multiple architectures are supplied, they are executed sequentially in the order provided.

Examples:

- `kernel build --86_64`
- `kernel run --aarch64`
- `kernel test --86_64`
- `kernel test --86_64 --aarch64`
- `kernel clean`

### Setup (so `kernel` works directly in shell)

Some WSL/Linux setups do not execute scripts in the current directory by default (or use mounts that block direct execution). Add this function to your shell config:

```bash
cat >> ~/.bashrc <<'EOF'

# Kernel helper
kernel() {
  bash "$HOME/projects/grovean/kernel" "$@"
}
EOF

source ~/.bashrc
```

Important: Verify the path to `kernel` is correct
