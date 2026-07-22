# Nuubot4 Rust Port Architecture Proposal

Status: BtRunner-through-control-Runtime implemented and proven; later stages
remain proposals.

## Problem

Nuubot3 has the required trading behavior and high-level flow, but its current
Python implementation mixes an active lifecycle rewrite with async tasks,
shared mutable objects, and locks. A previous Rust attempt also failed because
ownership was unclear.

Nuubot4 should preserve Nuubot3 behavior while making ownership direct enough
that Rust prevents concurrent mutation instead of coordinating it through
`Arc<Mutex<_>>`.

## Goal

Build one Rust crate containing two standalone programs:

```text
nuubot-runner <network> <bot_id>
nuubot-btrunner <sweep_id> <bot_id>
```

Each process receives identity only. It loads configuration, credentials,
stored Bot state, datastores, inputs, Clock, Runtime, and every owned child by
itself. A later CLI and Server will only create/configure identities and launch
or control these same programs.

Nuubot3's `server.py`, `api.py`, `botmgr.py`, and `sweepmgr.py` are the accepted
conceptual control-plane direction. They remain outside the Runner-owned
program lifecycle and Runtime-owned Bot lifecycle.

## Canonical Ownership

```text
Binary
`-- Runner / BtRunner          program lifecycle
    |-- environment/config
    |-- datastore/connections
    |-- inputs and clock
    |-- supervision/shutdown
    `-- Runtime                Bot lifecycle
        |-- Signaler
        |-- Risk
        `-- BotCycle
            `-- Executors
                `-- each Executor owns Vec<Account>
                    `-- Account
                        `-- Ledger
                            `-- Trades
                                `-- Orders
                                    `-- Fills
```

This tree is the primary organization rule. A component belongs under the
smallest owner whose lifecycle contains it. Changes that require cross-tree
mutable sharing or allow Runner to bypass Runtime need explicit redesign, not
another lock.

## Confirmed Source Contracts

- Runner and BtRunner own the complete program lifecycle and environment. They
  use the same Runtime contract.
- Runner uses WallClock and live inputs; BtRunner uses TickClock and replay.
- Each process owns one Bot and one event loop.
- Runtime owns and runs one Bot's lifecycle: Signaler, Risk, and zero or one
  BotCycle.
- BotCycle owns ordered Executors. Each Executor owns its `Vec<Account>`.
- Account owns one Ledger and exactly one selected exchange implementation.
- Ledger owns Trades; Trade owns Orders; Order owns Fills.
- Runtime reconciles complete Account truth before Risk and Executor decisions.
- Executor completion and Risk exit converge on one Runtime-owned stop route.
- Stop closes admission first, then unwinds direct children explicitly.

Use the current Nuubot3 `btrunner.py`, `runtime.py`, TickClock, Account, Ledger,
Trade, Order, Fill, and Simulator as the accepted behavioral source. The live
Runner is not yet the structural reference; its eventual Rust form should
align with BtRunner while substituting live environment resources. Review
WallClock before that stage.

## Legacy Rust Reference

The private [`ngudles69/nuutrader3`](https://github.com/ngudles69/nuutrader3)
repository is the earlier failed Rust implementation. Use it selectively:

- reuse its vendored Hyperliquid SDK, TLS patch, adapter evidence, and test
  fixtures where they still match the accepted Nuubot3 contract;
- do not copy its workspace decomposition, manager abstractions, shared
  Accounts, or per-callback mutex gates;
- treat its `Arc<Account>` and many `tick`, `timer`, `recon`, `state`, and
  lifecycle locks as failure evidence for Nuubot4's single-owner rule.

## Clarity Rule

A user should be able to read the execution path in order from the two binary
entrypoints. Keep the corresponding files and methods direct:

```text
binary main -> Runner/BtRunner -> Runtime -> BotCycle -> Executor -> Account
```

Names should describe domain ownership, lifecycle calls should remain visible,
and files should follow the ownership tree. Do not make the reader reconstruct
execution through registries, generic lifecycle traits, service containers,
callbacks stored across modules, or lock acquisition conventions.

## Current Cargo Layout

Do not create a `crate/` or `crates/` directory yet. The repository root is one
Cargo package and one library crate with two binaries.

```text
Cargo.toml
src/
  lib.rs
  bin/
    nuubot-btrunner.rs
  btrunner.rs
  runtime.rs
  botcycle.rs
  signaler.rs
  executor.rs
  risk.rs
  replay.rs
  config.rs
  setup.rs
  clock.rs
  datastore.rs
  market.rs
  logging.rs
  cloid.rs
  error.rs
```

Keep files flat while they remain short. Add a directory only when one current
component has enough real implementations to make the flat list harder to
read. Add a Cargo workspace only for a genuinely separate deployable or
reusable package.

## Composition And Lifecycle

The binaries contain program-specific argument parsing and lifecycle flow.
Shared `common/program.rs` helpers collect raw arguments and own the one final
error/exit mapping. The binaries construct Runner or BtRunner, which owns the
complete program lifecycle:
configuration, logging, datastores, stored state, inputs, Clock,
Runtime, supervision, shutdown, and result publication where applicable.

```text
binary main and common argument/exit helpers
-> binary parser
-> construct Runner or BtRunner from identity
-> Runner/BtRunner.init
-> Runner/BtRunner.start
-> Runner/BtRunner.run
-> Runner/BtRunner.stop
-> propagate the first failing phase to the shared process exit handler
```

Runner and BtRunner compose direct environment children:

```text
Runner
|-- SetupContext: Config, Credentials, PostgreSQL Store, stored Bot
|-- WallClock
|-- Bar input
|-- BBO input
`-- Runtime

BtRunner
|-- SetupContext: Config, read-only Sweep SQLite, stored Bot
|-- TickClock
|-- ReplayInput
|-- Runtime
`-- ResultPublisher
```

Runtime composes and executes one Bot. Runner drives Runtime's visible Bot
lifecycle, but never reaches through it into Accounts, Ledgers, Executors,
Orders, or Fills.

```text
program lifecycle                 Bot lifecycle
Runner/BtRunner.init        ->    Runtime.init
Runner/BtRunner.start       ->    Runtime.start
Runner/BtRunner.run loop    ->    Runtime input + bounded mainloop calls
Runner/BtRunner.stop        ->    Runtime.stop, then environment cleanup
```

## Rust Ownership Model

Use one owner for every mutable domain object. No parent/child reference cycles.

```text
Runtime
|-- Signaler
|-- Vec<Risk>
`-- Option<BotCycle>
    `-- Vec<Executor>
        `-- Vec<Account>
            |-- Ledger
            |   `-- Trade -> Order -> Fill
            `-- Exchange
                |-- Simulator
                `-- Hyperliquid
```

Each Executor creates and owns the Accounts it uses. One default Executor trait
method reconciles zero, one, or many owned Accounts and returns owned
`AccountSnapshot` values. BotCycle gathers those snapshots for Runtime's Risk
gate before Executor decisions. Runtime owns the sequence but never stores an
Account, Account ID, or Account reference. The canonical contract and Rust
shape are in [`wiki/recon.md`](../wiki/recon.md).

Domain children own values directly:

- `Ledger` owns `HashMap<TradeId, Trade>`.
- `Trade` owns `HashMap<Cloid, Order>`.
- `Order` owns `HashMap<FillKey, Fill>`.
- Children contain parent IDs for evidence but no parent references.
- Persistence rows are copies/snapshots, never long-lived references into the
  live domain tree.

### Current Ownership Audit

| Structure | Direct owner | Shared mutation | `Arc` | `Mutex` | Reason |
|---|---|---:|---:|---:|---|
| `BtRunner` | Binary stack | No | No | No | One synchronous program lifecycle |
| `SetupContext` | BtRunner | No | No | No | Immutable admitted infrastructure |
| Tick readers/windows | BtRunner | No | No | No | One serial replay cursor |
| Weekly `Vec<BboTick>` | BtRunner run scope | No | No | No | Loaded, consumed, then dropped |
| `TickClock` | BtRunner | No | No | No | One scheduling state |
| `Runtime` | BtRunner | No | No | No | One direct mutable child |
| Signaler/Risks/BotCycle | Runtime | No | No | No | Runtime sequences all calls |
| Executors | BotCycle | No | No | No | BotCycle calls them in order |
| Logger path values | Each component | No | No | No | Synchronous append; no shared handle |
| BBO/Bar/CLOID values | Passed by value | No | No | No | Trusted immutable data |

No current structure should gain `Arc` or `Mutex`. A future optional transport
queue may justify one synchronization primitive at that external ingress
boundary only. It does not justify shared domain ownership.

## Sync And Lock Policy

The canonical Runner and BtRunner control paths are synchronous. Recon is an
ordered truth barrier, so parallel domain mutation would be incorrect rather
than faster.

### BtRunner

BtRunner's hot replay path is synchronous and single-owner:

```text
read tick
-> Runtime.ingest_bbo
-> TickClock.advance
-> due Runtime.mainloop
```

It needs no tasks, channels, locks, `Arc`, or `Mutex`. SQLite, replay, Runtime,
and the later Simulator domain remain direct synchronous calls.

### Runner

Runner should also begin synchronous. The accepted design does not require a
WebSocket: normal venue calls, dirty state, and ordered recon can drive the Bot.
Do not add Tokio, a channel, or a transport task before a real live requirement
proves it necessary.

```text
optional transport -> owned event queue -> synchronous Runner -> &mut Runtime
```

There is no `Arc<Mutex<Runtime>>`, `Arc<Mutex<Account>>`, or domain-wide lock.
Runtime cannot receive concurrent `&mut self` calls, so Rust replaces Python's
`mainloop_in_progress`, `bbo_in_progress`, and Account-state lock ordering with
compile-time exclusivity.

The Runtime control pass remains explicitly sequential because its truth order
is required:

```text
consume due work
-> ask BotCycle to reconcile every Executor-owned Account
-> calculate coherent Ledger PnL/equity
-> evaluate Risk
-> call every Executor once
-> persist outcome
```

This is intentional domain ordering, not accidental lock serialization.
Independent socket/HTTP waits may overlap outside the domain owner. Channel
capacity and overflow behavior require a focused live-input design and proof;
do not silently drop or coalesce events.

If WebSocket becomes necessary later, isolate async work inside the transport
adapter. It may enqueue owned, boundary-validated BBO/UserEvent values. Runner
alone drains that queue at a visible point and mutates Runtime. That optional
queue is the only currently foreseeable place that may justify shared
synchronization; it never makes Runtime, Account, Ledger, Risk, or Executor
shared.

## Closed Variants, Not Frameworks

Use enums for configured closed sets:

```text
Signaler = Macross | EmaCross
Executor = Print | Trade
Risk = Balanced | MaxDrawdown
Exchange = Simulator | Hyperliquid
Store = Postgres | Sqlite
```

Each enum delegates directly with `match`. Add a trait only when two current
implementations genuinely require a dynamic boundary that an enum cannot
represent cleanly. Do not build registries, factories, dependency-injection
containers, generic lifecycle walkers, or a plugin system.

## Signaler And Calculations

Do not port Polars DataFrames into the trading domain by default. Macross,
EmaCross, ATR, timeframe alignment, and signal lookup can use typed `Vec<Bar>`,
rolling state, and direct structs. This gives Rust clear ownership and avoids
copying a Python research representation into the runtime.

Arrow stays inside the Parquet replay boundary and never enters the trading
domain. Only add Polars if a measured research/reporting workload needs it. The
Signaler owns typed timeframe state and emits typed Signal evidence.

## Exchange And Hyperliquid

Account owns one `Exchange` enum. Simulator and Hyperliquid are peers; neither
imports or falls back to the other. SDK-specific types stop at
`exchange/hyperliquid.rs`; Account receives Nuubot-owned request/response types.

Use the official
[`hyperliquid_rust_sdk`](https://github.com/hyperliquid-dex/hyperliquid-rust-sdk)
through a narrow Nuubot-owned adapter. SDK status checked 2026-07-22:

- crates.io still publishes 0.6.0; there is no newer released version;
- Nuutrader3 vendors the 0.6.0 release commit `67ee7fcb`;
- official `master` is 23 commits ahead at `aac75585`, but still declares
  version 0.6.0 and has not been published;
- official `master` adds BBO WebSocket support, CLOID on open Orders,
  active-asset data, newer response handling, and an `ethers` to `alloy`
  migration;
- official `master` still lacks `userFillsByTime`, `frontendOpenOrders`, Order
  lookup/modify by CLOID, and public order grouping for `normalTpsl` or
  `positionTpsl`;
- official `master` still enables native TLS, so Nuutrader3's Cargo-only rustls
  patch remains necessary;
- current open upstream issues include broken `market_close` behavior and an
  incorrect `class_transfer` action/signing path. Nuubot must not call either
  helper without its own proof.

Do not use crates.io 0.6.0 unchanged and do not follow floating `master`.
When the Hyperliquid stage begins, vendor the reviewed official commit, reapply
the two rustls dependency changes, and pin that source. Nuutrader3's SDK tests
and fixtures are evidence for the comparison, not permission to copy its wider
architecture.

The adapter proof must cover Nuubot's exact required contract:

- mainnet/testnet endpoint selection;
- wallet/address and agent-key signing;
- batch submit with exact CLOID and per-Order response parsing;
- cancel by CLOID;
- open Orders;
- unaggregated user Fills by inclusive time range;
- Account state/positions;
- exact Order status by CLOID;
- rate-limit and malformed/timeout behavior;
- clean HTTP/WebSocket shutdown.

If one required Info call is missing, add that direct REST call inside
`HyperliquidExchange`; do not introduce a second general SDK or fallback path.

## Config, Credentials, And Setup

Use `serde` plus `toml` for typed configuration. Validate the complete document
once at ingress, then pass trusted concrete types.

```text
Config::load(repo/config.toml)
Credentials::load(repo/workspace/config/credentials.toml)
nuubot_setup(identity, process_kind) -> SetupContext
```

`SetupContext` returns complete initialized process infrastructure or an error.
It does not start work. Runner/BtRunner owns the program lifecycle and
environment; Runtime owns the Bot lifecycle.

Credentials secrets must redact `Debug`/logging output. No environment fallback
or alternate configuration authority should be added.

## Datastore

Use direct datastore-specific functions. The current BtRunner uses one
read-only `rusqlite` connection during setup and drops it after loading the Bot
specification. Do not use an ORM.

Keep two concrete stores:

```text
PostgresStore   Runner runtime state
SqliteStore     BtRunner Sweep catalog and per-Bot database
```

Choose a synchronous PostgreSQL client when the Runner datastore stage begins,
unless measured transport requirements establish a real async need. SQL and row
mapping remain inside datastore functions; callers make one domain-level call.
Transactions remain short and explicit.

Schema changes are development hardcuts: recreate Nuubot4-owned schemas/files.
Do not add migrations, compatibility readers, dual writes, or Nuubot3 paths.

## Minimal Dependency Direction

Dependencies point inward:

```text
bin/runner, bin/btrunner
-> runner
-> runtime
-> signaler / risk / executor
-> account
-> ledger / trade / order / fill

exchange and datastore are boundary implementations used through Nuubot-owned
types; domain objects do not import binary, CLI, Server, or SDK types.
```

Current justified dependencies are boundary-specific: `serde`/`toml`/JSON for
config and Bot rows, `rusqlite` for the read-only Sweep copy, Arrow Parquet for
two projected columns, `chrono` for calendar windows, CSV for strict decoding,
and plain text errors propagated with `Result` and `?`. Add `rust_decimal`, a synchronous
PostgreSQL client, or the pinned Hyperliquid SDK only when their vertical stage
uses them. Tokio and tracing are not current defaults.

Do not add these before the vertical stage that uses them.

## Port Stages

1. **Stability harness** — complete: CSV and Parquet replay, real weekly owned
   windows, TickClock, and 200-process repetition for each loader.
2. **Project shape and BtRunner owner** — complete: harness moved behind
   `src/bin/nuubot-btrunner.rs`; add identity-only argument parsing,
   SetupContext, replay supervision, and visible lifecycle. No trading domain.
3. **Shared Runtime Bot lifecycle** — complete control slice: Runtime, temporary Macross/Print/
   Balanced components, Bot stop outcomes, and exact control ordering. Runner
   or BtRunner remains responsible for the surrounding program lifecycle.
4. **Canonical Signaler/Executor/Risk** — port real typed calculations and one
   complete decision path without Account internals.
5. **Simulator Account vertical slice** — Account, Ledger, Trade, Order, Fill,
   Simulator, SQLite evidence, and one accepted fixed Backtest parity result.
6. **BtRunner completion** — result publication, failure evidence, Sweep
   catalog behavior, and repeated Windows/Linux proof.
7. **Runner binary** — align its composition and lifecycle with BtRunner, then
   substitute PostgreSQL, reviewed WallClock, Runner-owned live inputs,
   recovery, shutdown, and the same Runtime.
8. **Hyperliquid adapter** — integrate the proven SDK and known fix, then prove
   live Account parity and shutdown behavior.
9. **CLI and Server** — later process-control layers that pass identity only.

Do not scaffold later stages early.

## Proof And Acceptance

Every stage keeps one owner-to-effect path runnable. Before progressing, compare
the applicable Nuubot3 evidence:

- admitted input count and timestamps;
- Clock callback count and order;
- Signal and BotCycle sequence;
- Executor entry/exit decisions;
- Trade, Order, and Fill identity/count/timestamps;
- Account/Ledger equity, fees, and PnL;
- Risk exit timing;
- graceful stop order and terminal state;
- persisted and published result.

For ownership specifically, reject a stage if:

- a domain object requires `Arc<Mutex<_>>`;
- a child contains a reference to its mutable parent;
- Runner reaches through Runtime;
- Runtime owns or supervises program-environment resources;
- Runtime stores an Account, Account ID, or Account reference;
- an Executor does not directly own the Accounts it uses;
- an async task mutates Runtime/Account state outside the single owner;
- stop order is hidden in generic type machinery;
- a Store or SDK type leaks into Trade/Order/Fill domain behavior.

## Non-Goals

- Line-by-line Python translation.
- One crate per component.
- Generic lifecycle traits.
- Actor framework or service container.
- Multi-threaded domain mutation.
- Server/CLI in the initial BtRunner port.
- Nuubot3 database writes or compatibility paths.
- Full trading port before each preceding vertical stage proves parity.

## Open Risks

1. WallClock needs review before the live Runner port; TickClock is accepted.
2. Live input buffering policy must preserve observable drop/gap behavior
   without reintroducing shared domain locks.
3. PostgreSQL and SQLite share domain behavior but not necessarily identical
   SQL; forcing one generic query layer may hide correctness differences.
4. Indicator parity must be proven numerically before replacing Polars-based
   calculations with direct Rust rolling state.
5. WebSocket is not required by the current Runner design. If later evidence
   requires it, decide per-Runner versus centralized ownership from measured
   connection/rate limits while keeping async mutation outside the Bot tree.
6. The official Hyperliquid SDK has unreleased fixes and remaining gaps. Pin a
   reviewed commit plus the rustls patch and prove only the adapter operations
   Nuubot actually uses.

## Recommendation

Review the current `.rs` names and visible L1 flow before adding descendants.
The next approved vertical stage should replace the temporary control behavior
with canonical Signaler/Executor/Risk behavior. Do not add Account, Venue,
Ledger, Runner, CLI, or Server until that stage is readable and proven.
