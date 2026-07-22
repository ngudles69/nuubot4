# HANDOFF

Last updated: 2026-07-22

## Focus

Review `src/btrunner.rs` next. Make its control flow and ownership as easy to
read as `src/bin/nuubot-btrunner.rs`.

## Current Status

- `src/bin/nuubot-btrunner.rs` now reads as `main -> program -> parser -> run`.
- `run()` visibly performs `init -> start -> run -> stop` with `?`.
- `BtRunner::stop()` owns successful summary logging.
- Program-fatal errors are complete text at their source, propagate with `?`,
  are logged once by the shared executable exit boundary, then exit.
- Bot information goes to its Bot log. Every error also goes to `errors.log`.
  Before a Bot logger exists, errors go only to `errors.log`.
- Log filename construction is owned by `common/logging.rs`.
- `wiki/coding/style.md` reserves Program Flow for control flow, ownership,
  lifecycle, loops, and intent-named calls. Mechanical detail belongs in its
  owning module or lower helper section.
- `wiki/logic/btrunner.md` documents the current control flow. Other logic pages
  remain partial or empty until their components are reviewed.

## Active Work

- No agents or delegated work.
- No active Nuubot process.
- No blocker.

## Files Changed

- BtRunner entrypoint, lifecycle, runtime children, setup, replay, config,
  datastore, CLOID, shared errors, logging, and program exit helpers.
- `Cargo.toml` and `Cargo.lock` remove the direct `thiserror` dependency.
- Coding rules, coding style, ownership, architecture plans, and logic pages.
- This handoff is included in the requested closeout commit on `main`.

## Proof

- `cargo fmt --all -- --check`: passed.
- `cargo check --bin nuubot-btrunner`: passed.
- Replay tests: 5/5 passed.
- CLOID test: 1/1 passed.
- Release build: passed.
- Invalid pre-identity input exited 1 and logged its exact text to `errors.log`.
- Missing stored Bot exited 1 and logged identical text to the Bot log and
  `errors.log`.
- Fresh-process proof: 10/10 passed, then 2/2 passed after final review.
- Latest proof log:
  `workspace/logs/nuubot4-rtest-s6-b9-2-20260722T154536Z.log`.
- Clippy is not clean because of the pre-existing `let_and_return` warning in
  `src/clock.rs`. That unrelated file was not changed.
- Full test suite was not run.

## Decisions

- Keep local `BotIdentity`; `identity` remains the variable name.
- Keep the current `logger(Some(bot_log_name(...)))` flow for now.
- Do not add separate Sweep and Live logger APIs before their design is agreed.
- Earlier lifecycle failure aborts upward. `stop()` runs only after successful
  `init`, `start`, and `run` in this executable.

## Next Action

Read `wiki/logic/btrunner.md` beside `src/btrunner.rs`. Review source order,
method names, comments, ownership, loops, and mechanical noise. Discuss first;
do not edit until the user confirms the review findings.
