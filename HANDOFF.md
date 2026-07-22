# HANDOFF

Last updated: 2026-07-22

## Focus

Transfer the root session into native Windows Herdr and prove the operator
workflow before resuming Nuubot4 development.

## Current Status

- Herdr `0.7.5-preview.2026-07-21-0f10e1453a7f` is installed at
  `C:\Users\PC\AppData\Local\Programs\Herdr\bin\herdr.exe` and that directory
  is on the user `PATH`.
- The Herdr Codex integration is current at v6. It installed
  `C:\Users\PC\.codex\herdr-agent-state.ps1` and updated Codex's `hooks.json`
  and `config.toml`.
- The named Herdr session `nuubot4` is running with native Windows Codex. The
  first launch used the home directory and correctly failed gate 1; relaunching
  Codex from `D:\rust\nuubot4` passed gate 1 and loaded the repository docs.
- The first in-Herdr `Hello.` test exposed a bootstrap-order failure: Codex read
  `soul.md` only after emitting generic commentary. It later produced the exact
  identity, current-state summary, confirmation boundary, and a correct
  self-diagnosis, but the strict first-response gate failed.
- The uncommitted startup revamp now gives `AGENTS.md` only the exact initial
  greeting and document order. `wiki/soul.md` remains the identity owner and
  now owns the ready response, dynamic synthesis, action-confirmation rule, and
  explicit `I told you before...` audit. The failed-session restart conflict was
  removed. The first ephemeral retest produced the exact greeting but ended the
  turn before reading startup documents; `AGENTS.md` now explicitly makes the
  greeting a commentary update and requires bootstrap to continue in the same
  turn. A second genuinely fresh ephemeral test passed the complete revised
  contract. The user then confirmed the same fresh `Hello.` startup works inside
  Herdr, so the revised startup contract is accepted.
- Nuubot4's 30-month BtRunner gate remains complete at 200/200 clean processes:
  78,883,200 ticks, 7,888,320 callbacks, and `stop_reason=replay_end`.
- BtRunner program outcomes now use symmetric `log_success` / `log_failure`
  paths. `simplelog` always writes the process file and mirrors it to the
  console only when local `config.toml` sets `logging.console = true`.
- The source-order convention is documented in `wiki/coding/style.md`: program
  flow and lifecycle first, context-sensitive domain logic second, and
  context-free mechanical helpers last. All 25 project Rust files were reviewed;
  six were reordered without changing behavior so lifecycle methods are easy to
  find. No new `common` module was created because no genuinely shared format or
  time helper currently exists.

## Herdr Acceptance Gates

1. Launch native Windows Codex in `D:\rust\nuubot4`.
2. Pass the `Hello.` startup contract.
3. Start, name, prompt, wait for, read, and re-prompt multiple agents.
4. Correctly detect blocked, working, and completed Codex states.
5. Preserve sessions through detach and reattach.
6. Isolate writing agents with Git worktrees.
7. Capture complete responses despite Codex's alternate-screen TUI.
8. Stop cleanly with no surviving processes.

Herdr is accepted only if every gate passes. Native Windows support is preview
beta, so do not treat successful installation as proof of the workflow.

## Files Changed

- Uncommitted repository files: `AGENTS.md`, `Cargo.toml`, `Cargo.lock`,
  `HANDOFF.md`, `src/bin/nuubot-btrunner.rs`, `src/botcycle.rs`,
  `src/common/logging.rs`, `src/config.rs`, `src/executor/observer.rs`,
  `src/replay/csv.rs`, `src/replay/parquet.rs`, `src/runtime.rs`, `src/setup.rs`,
  `wiki/soul.md`, `wiki/project.md`, `wiki/coding/index.md`,
  `wiki/coding/rules.md`, `wiki/coding/style.md`, `wiki/index.md`,
  `wiki/ownership.md`, and `wiki/rust-port.md`.
- Ignored local `config.toml` now contains `logging.console = true`.
- Machine-local Herdr and Codex integration files are outside this repository.

## Proof

- Herdr binary resolved and reported the installed preview version.
- `herdr integration status` reported `codex: current (v6)`.
- The user `PATH` contains Herdr's binary directory.
- Herdr session inventory reports named session `nuubot4` running and the
  default session stopped.
- Gate 1 passed after correcting the Codex working directory.
- Gate 2 failed under the prior startup contract. A fresh ephemeral read-only
  Codex retest passed the revised greeting, same-turn bootstrap, document order,
  ready response, dynamic project summary, and discussion boundary. The user
  then confirmed the fresh in-Herdr `Hello.` test works. Gates 1 and 2 pass;
  gates 3 through 8 have not been run.
- The passing ephemeral test exited zero. Codex `0.144.6` emitted non-fatal
  model-cache and unsupported PowerShell shell-snapshot warnings.
- Focused BtRunner proof passed: formatting, compile, Clippy (with the unrelated
  existing Clock lint excluded), console-on success, console-on failure, and
  console-off failure with zero stdout/stderr bytes while the file log grew.
- The source-order pass passed `cargo fmt --all -- --check`, the replay tests
  (5/5), the Cloid test (1/1), BtRunner `cargo check`, focused Clippy, and the
  release build. `bash rtest.sh 10 6 9` passed 10/10 runs: 79,488,000 ticks,
  7,948,800 callbacks, zero failed runs, and no surviving process. Proof log:
  `workspace/logs/nuubot4-rtest-s6-b9-10-20260722T103256Z.log`.
- No commit or push was performed.

## Next Action

Continue Herdr gate 3: start and name multiple agents, then prompt, wait for,
read, and re-prompt them. Continue gates 4 through 8 in order. After Herdr
passes, resume the one-question-at-a-time L1 review of
`src/bin/nuubot-btrunner.rs`, `src/btrunner.rs`, and `src/runtime.rs`.
