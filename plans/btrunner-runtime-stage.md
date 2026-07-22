# BtRunner Through Runtime Stage

Status: implemented, proven, and independently audited.

## Outcome

Build the identity-only `nuubot-btrunner <sweep_id> <bot_id>` program through
the accepted current Nuubot3 `runtime.py` control assembly. Keep Account,
Ledger, Trade, Order, Fill, Simulator, exchange, result publication, Runner,
CLI, and Server unwired.

## Behavioral Sources

- `D:\rust\nuubot3\nuubot\runner\btrunner.py`
- `D:\rust\nuubot3\nuubot\runtime\runtime.py`
- `D:\rust\nuubot3\nuubot\core\clock.py`
- `D:\rust\nuubot3\nuubot\signalers\macross.py`
- `D:\rust\nuubot3\nuubot\executor\printex.py`
- `D:\rust\nuubot3\nuubot\risk\balanced.py`
- `D:\rust\nuubot3\wiki\backtest.md`
- `D:\rust\nuubot3\wiki\runner-lifecycle.md`
- `D:\rust\nuubot3\wiki\clock.md`

The Python BtRunner still imports `runtime_sync.py`. The user explicitly chose
current `runtime.py` for this Rust stage, so that stale import is not copied.

## Ownership Invariant

```text
Binary
`-- BtRunner
    |-- SetupContext
    |-- TickClock
    `-- Runtime
        |-- MacrossSignaler
        |-- Vec<BalancedRisk>
        `-- ControlBotCycle
            `-- Vec<ObserverExecutor>
```

- Every mutable object has one owner.
- No `Arc`, `Rc`, `Mutex`, `RwLock`, channel, task, or child-to-parent reference.
- BtRunner owns the program lifecycle and calls Runtime's Bot lifecycle.
- `nuubot_setup` owns common infrastructure admission and returns ready inputs.
- `nuubot_setup` returns one owned `SetupContext` containing ready config,
  validated Bot specification, and the shared Bot logger. Its read-only
  SweepStore is dropped after Bot admission because execution does not use it.
- Replay is not a lifecycle object. A public iterator function hides discovery
  and decoding, validates rows once, and emits owned `BboTick` values.
- CSV and Parquet converge before Runtime; downstream behavior is identical.
- Runtime owns no environment path, database connection, or file reader.
- BtRunner contains no TOML, SQLite, schema, or file-discovery mechanics.
- Accounts are not represented by placeholders in this stage.

## Files

```text
Cargo.toml
.gitignore
config.toml
src/lib.rs
src/bin/nuubot-btrunner.rs
src/btrunner.rs
src/setup.rs
src/config.rs
src/datastore.rs
src/replay.rs
src/clock.rs
src/logging.rs
src/market.rs
src/runtime.rs
src/botcycle.rs
src/signaler.rs
src/risk.rs
src/executor.rs
rtest.sh
workspace/datastore/nuubot4_sweeps.db
```

Do not create a workspace, sub-crate, lifecycle trait, factory framework, or
generic datastore layer.

## Config Contract

Root `config.toml` is the non-secret installation authority. It owns environment
selection, paths, BtRunner loader, Runtime control composition, and future
Simulator defaults. `config.rs` also owns a separate credentials loader for
`workspace/config/credentials.toml`, but BtRunner does not call it because a
backtest needs no secrets.

The loader is a closed enum with current values `csv` and `parquet`. A future
real loader adds one enum variant and one decoding function. Unknown values
fail config admission.

## Lifecycle

```text
main
-> parse positive Sweep/Bot IDs
-> BtRunner::new
-> init: call nuubot_setup, create TickClock and Runtime subtree
-> start: start Runtime and register its configured timer
-> run: call replay iterator, ingest BBO, advance Clock
-> stop: close Runtime admission, unwind BotCycle/Executors, drop SetupContext
-> preserve first failure and return process exit
```

Runtime preserves current control order:

```text
count mainloop
-> assess every Risk
-> latch Risk stop
-> close active cycle when stopping
-> otherwise call every active Executor once
-> close completed cycle
-> stop at max_cycles or create one fresh cycle
```

## Root-Cause Gate

Complete path:

```text
identity -> config validation -> SQLite Bot row -> Bot config validation
-> file selection -> CSV/Parquet boundary validation -> BboTick
-> Runtime ingest -> TickClock -> Runtime mainloop -> Runtime outcome -> stop
```

All deterministic validation happens before Runtime starts where possible.
Per-row numeric/timestamp validation happens before each tick enters Runtime.
This stage performs no datastore mutation or external trading call.

The binary parser creates one `BotIdentity`. `run()` creates the named Bot log
before setup validates the stored Bot. BtRunner, Runtime, Signaler, Risk,
BotCycle, and Executors share that single ordered process log. The shared
program exit helper logs the final error once.

Failure classes:

| Class | Required behavior |
|---|---|
| Missing config, database, Bot row, or data file | Fail before start with exact path/identity |
| Wrong config scalar or unknown variant | Fail config admission |
| Empty path, symbol, or invalid date range | Fail Bot/config admission |
| Missing/wrong CSV or Parquet column type | Fail before admitting that row/batch |
| Non-finite/non-positive price | Reject before Runtime mutation |
| Backward timestamp | TickClock fails immediately |
| Empty replay | Fail run; still stop initialized children |
| Contradictory MA periods or event limits | Fail Runtime config admission |
| Any lifecycle failure | Propagate directly to shared `main`; log once and exit |

The default error policy is fail loud and fail fast. Every error propagates to
the shared binary `main`, is logged once, and exits nonzero. `stop()` runs only
after successful `init`, `start`, and `run`. There are no retries, skipped
records, fallback loaders, partial-success outcomes, or secondary cleanup error
paths unless a specific recovery contract is later approved.

## Readability Contract

- Rust `///` documentation precedes each public or non-obvious function.
- Every meaningful logical block begins with a three-to-four-word intent
  comment that allows comment-only skimming.
- Do not comment obvious assignments or syntax.
- Lifecycle methods remain in execution order.
- Helpers isolate actual boundaries only: config, datastore, replay decoding,
  Clock, and Runtime-owned components.
- The visible BtRunner path reads as setup, create Clock, create Runtime, load
  ticks, replay ticks, and stop on error or end.
- Utility modules expose small task-level public functions; their parsing,
  validation, schema, and file-discovery mechanics remain internal.
- Domain modules contain domain behavior only. Config consumption is one typed
  field or getter expression, and each datastore operation is one domain-level
  call at its caller. SQL, transactions, retries, row mapping, and storage
  validation remain inside the datastore module.
- Infrastructure helpers hide mechanics, not domain sequence: the caller must
  still visibly show the ordered business and lifecycle operations.
- Cross-owner calls express one domain intent in one line: BtRunner calls only
  Runtime's public ingress/lifecycle methods, Account later calls one Venue
  operation, and domain objects later call one datastore operation. A parent
  never reaches through a child to operate its descendants.

## Proof

1. `cargo fmt --check`.
2. `cargo check`.
3. `cargo build --release`.
4. One CSV operator run for Sweep 6 Bot 9.
5. One Parquet operator run for the same identity and range.
6. Compare exact tick count, first/last timestamp, Runtime mainloop count,
   completed cycles, stop reason, and ordered lifecycle output.
7. Run exactly 200 fresh release-binary processes for CSV, then exactly 200 for
   Parquet, with the same Sweep 6 Bot 9 range. Preserve every run's loader,
   elapsed time, exit status, exact tick/range/callback result, and failure
   diagnostics. Stop that loader's gate on its first failure.
8. Perform an implementation audit and ownership architecture review.

Completed evidence:

- CSV: 200/200, average 1.053 seconds, zero failures.
- Parquet: 200/200, average 0.558 seconds, zero failures.
- Every run: 7,948,800 ticks, 794,880 callbacks, exact timestamps.
- Final implementation and ownership audit: PASS.

## 30-Month Stability Extension (2026-07-22)

- All 30 Parquet files and 78,883,200 rows are complete for the half-open range
  `2023-12-01..2026-06-01`; their represented seconds are continuous.
- Raw close timestamps use two valid precisions: `2023-12..2024-12` end in
  `999000us`, while `2025-01..2026-05` end in `999999us`. This is not missing
  data.
- Fix the shared `src/replay.rs` admission/sequence boundary used by CSV and
  Parquet. Keep raw range filtering unchanged.
- Require every accepted one-second candle close to fall within the final
  1,000 microseconds before its next whole-second boundary. Normalize it to
  that boundary's `ts_ms`, then require admitted timestamps to advance by
  exactly 1,000ms.
- Regression proof covers both precisions, the precision transition,
  consecutive timestamps, duplicates, gaps, invalid fractions, and overflow.
- This corrects the latent pre-2025 Nuubot3 `(close + 1) // 1000` defect without
  modifying Nuubot3 or the shared source data.
- After focused proof, force a real early Runtime stop with `max_cycles=1` and
  prove that `rtest.sh` rejects it. Restore `max_cycles`, raise Observer
  `max_ticks`, prove one standalone process with exact counts and timestamps,
  then run the 200-process stability gate.
- Expect exactly 78,883,200 ticks, 7,888,320 callbacks, first UTC timestamp
  `1701388801000`, and last UTC timestamp `1780272000000`.
- Create an isolated Nuubot4 test Bot identity in the copied SQLite database;
  do not modify Nuubot3 or shared data.
- Retain `cargo fmt` and `cargo fmt --check`, build release, and prove one
  standalone process.
- Run exactly 200 fresh processes through `rtest.sh`, with a one-second pause,
  120-second timeout, first-failure stop, and dated log preservation.
- `rtest.sh` counts PASS only when the process exits zero and its summary
  contains `stop_reason=replay_end`. BtRunner internally enforces the exact
  tick count, callback count, first timestamp, and last timestamp for every
  complete replay.
- A premature Runtime stop is not a completed stability replay.
- Observer `max_ticks` completes and replaces one BotCycle; it does not stop
  Runtime. The initial 10,000,000 cap therefore produced seven completed cycles
  and still reached replay end. Raising it to 100,000,000 keeps this stability
  run inside one BotCycle.
- Negative gate proof: temporary `max_cycles=1` stopped at 10,000,001 ticks
  with `stop_reason=max_cycles`; `rtest.sh` rejected it. Log:
  `workspace/logs/nuubot4-rtest-s8-b12-1-20260722T065518Z.log`.
- Standalone and one-process positive proofs passed with the exact counts and
  UTC timestamps above, zero completed cycles, and `stop_reason=replay_end`.
  Harness log: `workspace/logs/nuubot4-rtest-s8-b12-1-20260722T065546Z.log`.
- Plan-audit v1's early-stop prediction was rejected by runtime proof:
  `max_ticks` rotates BotCycle rather than stopping Runtime. Its timestamp
  values were also eight hours early; the UTC values above are the source-data
  boundaries. The v2 requirement to reject any non-`replay_end` summary remains
  accepted and is proven by the forced `max_cycles` stop.
- Final 30-month Parquet gate: 200/200 fresh processes passed; every run emitted
  the exact counts, UTC timestamps, zero completed cycles, and
  `stop_reason=replay_end`. Average 5.085s, minimum 5.020s, maximum 5.195s.
  Log: `workspace/logs/nuubot4-rtest-s8-b12-200-20260722T065614Z.log`.

## Deliberate Deferrals

- No Account/Ledger/Simulator types or fake handles.
- The later boundary is fixed: Account is the Bot-facing interface to a Venue;
  Simulator and Hyperliquid are equivalent Venue implementations; Ledger
  records Bot-to-Venue transactions as Trades containing Orders containing
  Fills.
- No Bars are loaded in this slice; Macross remains initialized but receives no
  Bar until the Bar-ingress stage.
- No datastore writes or result publication.
- No credentials file is created or read.
- No live Runner binary implementation.
