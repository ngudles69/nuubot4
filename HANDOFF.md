# HANDOFF

Last updated: 2026-07-22

## Focus

Review the readable Rust names and L1 flow before porting canonical trading
behavior below the temporary control Runtime.

## Current State

- One Cargo package with standalone binary `nuubot-btrunner <sweep_id> <bot_id>`.
- Current identity proof: Sweep 6 Bot 9, 2026-03-01 through 2026-06-01.
- `config.toml` is the non-secret authority; current loader is `parquet`.
- Nuubot4 reads `workspace/datastore/nuubot4_sweeps.db` and shared read-only
  market data under `D:\workspace\data`.
- The synchronous path is `BtRunner -> Runtime -> BotCycle -> Executors`;
  Runtime also owns Signaler and Risks.
- No Account/Ledger/trading stack, Simulator, live Runner, async task, channel,
  `Arc`, or `Mutex` is wired yet.
- Each calendar week is one owned `Vec<BboTick>`; external rows are validated
  once, then trusted. Failures preserve the first cause, unwind initialized
  children, log fatal evidence, and exit nonzero.
- `main` is published at `github.com/ngudles69/nuubot4`.

## Ownership Decision

Every current mutable structure has one synchronous owner; none needs `Arc` or
`Mutex`. A future WebSocket transport may own a synchronized event queue only
at its external boundary and must not share mutable Bot state.

[Recon ownership](wiki/recon.md) is canonical:

```text
Runtime -> BotCycle -> Executors -> each Executor owns Vec<Account>
                                  -> Venue
                                  -> Ledger -> Trades -> Orders -> Fills
```

Runtime owns the recon/Risk/decision sequence but no Accounts. BotCycle asks
each Executor to reconcile its owned Accounts and returns owned
`AccountSnapshot` values to Runtime for Risk.

## Authoritative Proof

- `cargo fmt --check`, `cargo check`, `cargo build --release`: passed.
- `cargo test`: 1 passed across 3 suites.
- Independent plan and implementation/ownership audits: passed after fixes.
- CSV gate: 200/200 clean processes; 7,948,800 ticks and 794,880 callbacks per
  run; average 1.053 s; log
  `workspace/logs/nuubot4-rtest-s6-b9-200-20260721T174439Z.log`.
- Parquet gate: 200/200 clean processes; identical counts; average 0.558 s;
  log `workspace/logs/nuubot4-rtest-s6-b9-200-20260721T175157Z.log`.
- Each process was fresh with a one-second pause.

## Collaboration Closeout

- `AGENTS.md` now requires startup reads of this handoff, `wiki/user.md`, and
  `wiki/soul.md`, and records the controller/delegation workflow.
- `wiki/user.md` holds the user working profile; `wiki/soul.md` holds the
  chief-of-staff contract; `wiki/index.md` links both.
- No delegated work is pending. No blockers.
- This docs batch was checked by content inspection and `git diff --check`.
  Rust tests were not rerun because Rust code did not change.
- User authorized committing all current changes and pushing this turn.

## Next Action

Review filenames, public function names, and visible flow in
`src/bin/nuubot-btrunner.rs`, `src/btrunner.rs`, and `src/runtime.rs`. Rename or
simplify before porting canonical Signaler/Executor/Risk behavior. Do not wire
Account/Venue/Ledger or live Runner yet.
