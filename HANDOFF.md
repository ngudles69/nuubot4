# HANDOFF

Last updated: 2026-07-23

## Focus

Refactor BtRunner and Runtime lifecycle telemetry so each component owns and
logs its own statistics during stop.

## Current Status

- `src/bin/nuubot-btrunner.rs` now reads as `main -> program -> parser -> run`.
- `run()` performs `init -> start -> run`, always attempts `stop()`, and
  returns the replay or teardown error.
- Runtime no longer exports an outcome or summary to BtRunner.
- Each component logs its own stop statistics; parents receive only control
  results and errors.
- The CLI calls stop even when replay fails, preserving terminal evidence.
- Program-fatal errors are complete text at their source, propagate with `?`,
  are logged once by the shared executable exit boundary, then exit.
- Bot information goes to its Bot log. Every error also goes to `errors.log`.
  Before a Bot logger exists, errors go only to `errors.log`.
- Log filename construction is owned by `common/logging.rs`.
- `wiki/coding/style.md` reserves Program Flow for control flow, ownership,
  lifecycle, loops, and intent-named calls. Mechanical detail belongs in its
  owning module or lower helper section.
- `wiki/logic/btrunner.md`, `signaler.md`, `risk.md`, and `executor.md` document
  the current ownership and stop-telemetry flow.

## Active Work

- No agents or delegated work.
- No active Nuubot process.
- No blocker.
- `src/runtime.rs` now has explicit placeholders for the future signal-driven
  BotCycle shape. Signal generation and deferred cycle creation remain pending
  until the Signaler is implemented.
- Runtime start failures now attempt child cleanup and preserve the original
  error; child stop failures mark Runtime failed while all children are still
  attempted.

## Files Changed

- BtRunner entrypoint, lifecycle, clock, TickReader, Runtime, BotCycle,
  Signaler, Risk, Executor, setup, and replay.
- `rtest.sh`, current lifecycle/replay docs, ownership docs, coding rules,
  coding style, and `wiki/abbreviations.md`.

## Proof

- `cargo fmt --all -- --check`: passed.
- `cargo check --bin nuubot-btrunner`: passed.
- Replay tests: 5/5 passed.
- CLOID test: 1/1 passed.
- Release build: passed.
- Invalid pre-identity input exited 1 and logged its exact text to `errors.log`.
- Missing stored Bot exited 1 and logged identical text to the Bot log and
  `errors.log`.
- Fresh-process proof: 2/2 passed, then 1/1 passed after the final failure-state
  adjustment.
- Two additional fresh-process runs passed after the Runtime cleanup changes:
  `workspace/logs/nuubot4-rtest-s6-b9-1-20260722T180416Z.log` and
  `workspace/logs/nuubot4-rtest-s6-b9-1-20260722T180417Z.log`.
- Release binary rebuilt after lifecycle logging changes; an additional run
  passed with `runtime: init()`, `runtime: start()`, and `runtime: stop()` in
  the generated log:
  `workspace/logs/nuubot4-rtest-s6-b9-1-20260722T180636Z.log`.
- Latest proof logs:
  `workspace/logs/nuubot4-rtest-s6-b9-2-20260722T172255Z.log` and
  `workspace/logs/nuubot4-rtest-s6-b9-1-20260722T172424Z.log`.
- The Bot log contains terminal records for Executor, BotCycle, Risk, Signaler,
  Runtime, TickReader, TickClock, and BtRunner.
- Full test suite was not run.

## Decisions

- Keep local `BotIdentity`; `identity` remains the variable name.
- Keep the current `logger(Some(bot_log_name(...)))` flow for now.
- Do not add separate Sweep and Live logger APIs before their design is agreed.
- Earlier lifecycle failure aborts upward. After start, the CLI always calls
  `stop()` even when `run()` fails.

## Next Action

Implement the Signaler contract, then add deferred BotCycle creation and test
that a signal on bar N can only enter on a later tick/bar.
