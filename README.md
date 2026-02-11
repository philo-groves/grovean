# Grovean

## How to Build

Cross-compiling is supported by LLVM, allowing developers to build for non-native architectures.

`cargo build --target linker/<arch>-grovean.json`

#### Build with x86_64

`cargo build --target linker/x86_64-grovean.json`

#### Build with aarch64

`cargo build --target linker/aarch64-grovean.json`

## How to run

Running is executed through the `k1` crate. As a command line utility, it must be be installed on your system (`cargo install k1`).

#### Run with x86_64

`cargo run --target linker/x86_64-grovean.json`

#### Run with aarch64

`cargo run --target linker/aarch64-grovean.json`

## How to Test

Testing is conducted through the `kunit` crate.

`cargo test --target linker/<arch>-grovean.json`

#### Test with x86_64

`cargo test --target linker/x86_64-grovean.json`

#### Test with aarch64

`cargo test --target linker/aarch64-grovean.json`

#### Test Specific Crate

To only test a single crate, instead of the entire workspace, pass the `-p` flag:

`cargo test -p <some-crate> --target linker/<arch>-grovean.json`

## `grovean` shorthand script (Linux/WSL)

This repository includes a root-level `grovean` script that wraps cargo commands with shorthand architecture flags.

Supported commands:

- `build`
- `run`
- `test`

Supported architecture flags:

- `--86_64` (alias: `--x86_64`)
- `--aarch64`

At least one architecture must be supplied. If multiple architectures are supplied, they are executed sequentially in the order provided.

Examples:

- `grovean build --86_64`
- `grovean run --aarch64`
- `grovean test --86_64`
- `grovean test --86_64 --aarch64`

### Setup (so `grovean` works directly in shell)

Some WSL/Linux setups do not execute scripts in the current directory by default (or use mounts that block direct execution). Add this function to your shell config:

```bash
cat >> ~/.bashrc <<'EOF'

# Grovean helper
grovean() {
  bash "$HOME/Projects/grovean/grovean" "$@"
}
EOF

source ~/.bashrc
```

Important: Verify the path to `grovean` is correct