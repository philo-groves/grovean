# AGENTS.md

## Purpose
Operational context and rules for coding agents working in this repository.
Keep this file current as source-of-truth guidance for future sessions.

## Command Execution Rules
- Use the `kernel` helper for normal build/run/test flows and cleanup (`kernel clean`).
- Keep architecture explicit on every run/test command.
- If `kernel` is not on PATH or not executable in the current shell, invoke it as `bash ~/projects/grovean/kernel ...`.

## Test Flow Notes
- Standard workspace test flow: `kernel clean` then `kernel test --x86_64` (or `--aarch64` as needed).
- After `kernel test`, inspect normalized outputs under `.k1/testing/*.jsonl`.
- Also check `.k1/logs/k1-test-*.log` when diagnosing missing or incomplete normalized test records.
- Treat explicit `failed > 0` or test entries with non-pass results as failures.
- If a `test_group` reports a non-zero `test_count` but lacks per-test lines in `.k1/testing`, flag it as a possible test-output normalization/capture issue even if runner exits successfully.

## `kernel` Script (concise usage)
- Usage:
  - `kernel <build|run|test> [cargo-args...] <--86_64|--x86_64|--aarch64> [...]`
  - `kernel clean [k1-args...]`
- Commands:
  - `build`
  - `run`
  - `test`
  - `clean`
- Arch flags:
  - `--86_64`
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
- Do not run plain `cargo` commands for kernel verification; always pass a valid kernel target JSON.

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
