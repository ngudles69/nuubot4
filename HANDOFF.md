# HANDOFF

Last updated: 2026-07-22

## Focus

Review the readable Rust names and L1 flow before porting the canonical trading
behavior below the temporary control Runtime.

## Current State

- One Cargo package; no `crate/` or `crates/` directory.
- Standalone binary: `nuubot-btrunner <sweep_id> <bot_id>`.
- Current identity proof: Sweep 6 Bot 9, 2026-03-01 through 2026-06-01.
- `config.toml` is the non-secret authority. Current loader is `parquet`.
- Credentials remain separate and ignored; BtRunner does not load them.
- Nuubot4 reads its copied Sweep database at
  `workspace/datastore/nuubot4_sweeps.db` and shared read-only market data under
  `D:\workspace\data`.
- Actual path is synchronous:

  ```text
  binary
  -> BtRunner: setup, Clock, Runtime, weekly replay, stop
  -> Runtime: Signaler, Risks, BotCycle
  -> BotCycle: Executors
  ```

- No Account, Ledger, Trade, Order, Fill, Simulator, Hyperliquid, live Runner,
  Server, CLI, WebSocket, async task, channel, `Arc`, or `Mutex` is wired.
- Logger preserves one generic pre-config log and one ordered Bot identity log.
- `cloid.rs` preserves the Nuubot3 128-bit layout with only `encode_cloid` and
  `decode_cloid` operations.
- Every calendar week is loaded into one owned `Vec<BboTick>`, consumed, and
  dropped. Monthly files are streamed once.
- External CSV/Parquet rows are validated once into trusted `BboTick` values.
  Internal code does not revalidate or repair admitted data.
- Failures propagate to the binary, preserve the first cause, unwind initialized
  children, log fatal evidence, and exit nonzero.
- Git repository: `main`, published to `github.com/ngudles69/nuubot4`.

## Proof

- `cargo fmt --check`: passed.
- `cargo check`: passed.
- `cargo test`: 1 passed across 3 suites.
- `cargo build --release`: passed.
- Independent plan audit: PASS after corrections.
- Independent implementation and ownership audit: PASS after corrections.
- CSV three-month gate: 200/200 passed, 0 failed.
  - exact ticks each run: 7,948,800
  - exact callbacks each run: 794,880
  - average process time: 1.053 s
  - min/max: 1.040/1.077 s
  - log: `workspace/logs/nuubot4-rtest-s6-b9-200-20260721T174439Z.log`
- Parquet three-month gate: 200/200 passed, 0 failed.
  - exact ticks each run: 7,948,800
  - exact callbacks each run: 794,880
  - average process time: 0.558 s
  - min/max: 0.548/0.578 s
  - log: `workspace/logs/nuubot4-rtest-s6-b9-200-20260721T175157Z.log`
- Each process was fresh and separated by a one-second pause.
- An earlier post-audit-invalid CSV gate was deliberately stopped after 65/65;
  it is retained but is not part of the authoritative 200/200 result.

## Ownership Decision

Every current mutable structure has one synchronous owner. No current structure
needs `Arc` or `Mutex`. A future optional WebSocket transport may justify a
synchronized event queue at that external boundary only; it must never share or
mutate Runtime, Account, Ledger, Risk, Executor, or other Bot state.

The accepted Recon ownership is canonical in [wiki/recon.md](wiki/recon.md):

```text
Runtime
-> BotCycle
   -> Executors
      -> each Executor owns Vec<Account>
         -> Venue
         -> Ledger -> Trades -> Orders -> Fills
```

Runtime owns the recon/Risk/decision sequence but owns no Account list,
`AccountBook`, `AccountId`, or Account reference. BotCycle asks each Executor to
run the default trait reconciliation over its owned Accounts and returns owned
`AccountSnapshot` values to Runtime for Risk.

## Coding Authority

- Read every page under `wiki/coding/**` before coding.
- Calls cross ownership boundaries at the level of intent.
- Config reads and datastore/Venue operations are one task-level call in their
  caller; mechanics remain inside the owning boundary module.
- Validate external data once, then trust internal Rust types.
- Fail loud and fail fast; add recovery only for a later explicitly accepted
  failure class.
- Use short intent comments and `///` documentation so comment-only reading
  exposes the L1 flow.
- `Arc`, `Rc`, `Mutex`, `RwLock`, and `RefCell` are prohibited unless the user
  explicitly approves one concrete use and its reason and ownership effects are
  recorded in the owning wiki before implementation.

## Next Action

User reviews the `.rs` filenames, public function names, and visible flows in
`src/bin/nuubot-btrunner.rs`, `src/btrunner.rs`, and `src/runtime.rs`. Rename or
simplify before porting canonical Signaler/Executor/Risk behavior. Do not wire
Account/Venue/Ledger or live Runner yet.
