# HANDOFF

Last updated: 2026-07-22

## Focus

The 30-month BtRunner stability extension is complete. Resume the readable L1
review before porting canonical trading behavior below the temporary Runtime.

## Current Status

- `nuubot-btrunner 8 12` replays Parquet from `2023-12-01` through
  `2026-06-01` in one standalone process.
- Sweep 8/Bot 12 and its 30-month range live only in the ignored Nuubot4 SQLite
  workspace. Nuubot3 and shared market data were not changed.
- Ignored local `config.toml` has `max_cycles=999999` and Observer
  `max_ticks=100000000` so this gate stays inside one BotCycle.
- Pre-2025 closes end in `999000us`; 2025 onward ends in `999999us`. Both are
  admitted only in the final 1,000 microseconds and normalized to the next UTC
  second at the shared CSV/Parquet boundary.
- `rtest.sh` now requires exit zero and `stop_reason=replay_end` for every PASS.
- No Nuubot process or delegated work remains. No blockers.

## Decisions

- Keep one timestamp path: validate raw close boundary, normalize once, then
  require admitted timestamps to advance exactly 1,000ms. No legacy fallback.
- Observer `max_ticks` completes and replaces BotCycle; only a Runtime stop such
  as `max_cycles` ends replay early.
- Plan-audit v1's early-stop prediction and timestamps were wrong and are
  explicitly superseded in [the active plan](plans/btrunner-runtime-stage.md).

## Proof

- `cargo fmt`, `cargo fmt --check`, five focused replay tests, `cargo check`,
  release build, and `git diff --check`: passed.
- Forced early stop: temporary `max_cycles=1` produced `stop_reason=max_cycles`;
  `rtest.sh` rejected it. Log:
  `workspace/logs/nuubot4-rtest-s8-b12-1-20260722T065518Z.log`.
- Standalone and one-process gates passed with 78,883,200 ticks, 7,888,320
  callbacks, timestamps `1701388801000..1780272000000`, zero completed cycles,
  and `stop_reason=replay_end`.
- Final gate: 200/200 fresh processes passed; average 5.085s, minimum 5.020s,
  maximum 5.195s. Log:
  `workspace/logs/nuubot4-rtest-s8-b12-200-20260722T065614Z.log`.
- [Implementation audit v2](audits/btrunner-30m-implementation-audit-v2.md):
  PASS after comment-only readability fixes.
- Full `cargo test` was not run; repo rules require focused proof unless the
  user approves the full suite.

## Files Changed

- Replay admission: `src/replay.rs`, `src/replay/csv.rs`,
  `src/replay/parquet.rs`.
- Harness and comments: `rtest.sh`, `src/bin/nuubot-btrunner.rs`.
- Durable evidence: `plans/btrunner-runtime-stage.md`, `wiki/rust-port.md`, and
  `audits/btrunner-30m-*.md`.
- Local ignored state: `config.toml`, Sweep 8/Bot 12 database rows, proof logs.

## Next Action

Continue the one-question-at-a-time review of `src/bin/nuubot-btrunner.rs`, then
`src/btrunner.rs` and `src/runtime.rs`. The `and_then` question is resolved:
`run()` executes only when `start()` succeeds, while `stop()` still runs and the
first lifecycle failure is preserved.
