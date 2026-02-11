# AGENTS.md

## Purpose
Operational context and rules for coding agents working in this repository.
Keep this file current as source-of-truth guidance for future sessions.

## Environment
- Host OS for the coding agent: Windows 11.
- Repository runtime/build environment: WSL Ubuntu.
- Repo path in WSL: `~/Projects/grovean`.
- Always run project commands through WSL shell context (not native Windows toolchain).

## Command Execution Rules
- Prefer running from repo root in WSL: `~/Projects/grovean`.
- Use the `kernel` helper for normal build/run/test flows and cleanup (`kernel clean`).
- Keep architecture explicit on every run/test command.

## `kernel` Script (concise usage)
- Usage:
  - `kernel <build|run|test> [cargo-args...] <--86_64|--x86_64|--aarch64> [...]`
  - `kernel clean [k1-args...]`
- Commands:
  - `build`
  - `run`
  - `test`
  - `clean` (passthrough to `k1 clean`)
- Arch flags:
  - `--86_64` (alias: `--x86_64`)
  - `--aarch64`
- Rules:
  - At least one architecture flag is required for `build`, `run`, and `test`.
  - Multiple architectures run sequentially in the order provided.
- Examples:
  - `kernel build --86_64`
  - `kernel run --aarch64`
  - `kernel test --86_64 --aarch64`
  - `kernel test -p extended-crate --86_64`
  - `kernel clean`

## Direct Cargo Fallback
- Build: `cargo build --target linker/<arch>-grovean.json`
- Check: `cargo check --target linker/<arch>-grovean.json`
- Test: `cargo test --target linker/<arch>-grovean.json`
- Do not run plain `cargo check` for kernel verification; always pass a valid kernel target JSON.
- Runner for `target_os = "none"` is configured as `k1` in `.cargo/config.toml`.

## CI Notes
- GitHub Actions uses shared `kunit` workflow (`.github/workflows/test.yml`).
- Current tested targets:
  - `linker/x86_64-grovean.json`
  - `linker/aarch64-grovean.json`

## Workspace Notes
- Workspace members:
  - `crates/grovean`
  - `crates/basic-crate`
  - `crates/extended-crate`

## Kernel Development Priorities (current)
- Preferred sequence:
  1. Normalize boot memory map.
  2. Implement physical frame allocator.
  3. Implement paging manager on top of owned frames.
  4. Add/expand kernel heap integration.
- Do not rely long-term on bootloader-provided page tables beyond early boot.

## Engineering Guardrails
- Cross-arch parity: any memory-management API change should compile for both x86_64 and aarch64.
- Test expectation: run at least one target locally before finalizing major kernel-memory changes.
- Boot invariant: initialize and normalize Limine memory map (`grovean::memory::init`) before allocator or paging setup.

## Auto-Update Protocol (required)
When any important rule or decision changes, update this file in the same change set.
Treat updates as mandatory, not optional.

Update triggers:
- New build/test/run command conventions.
- Target/toolchain/runner/linker changes.
- CI workflow behavior changes.
- Architecture support changes.
- New kernel invariants (memory, paging, interrupts, allocator contracts).
- Any repeated instruction that would help future agent context.

Update style:
- Keep entries concise and actionable.
