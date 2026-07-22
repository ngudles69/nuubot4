# Ownership and Project Structure

This page is the canonical Nuubot4 ownership and module map. Nuubot3 remains
the behavioral source, but [Recon](recon.md) owns the approved Rust Account
ownership difference.

## Process Ownership

```text
RunnerService                         live control boundary, when ported
|-- ProcessStore
|-- RunnerControl                    owns no Runner reference
`-- Runner
    |-- WallClock
    |-- BarFeed
    |-- optional BboFeed
    `-- Runtime

BtRunner
|-- TickClock
|-- TickReader
|-- ResultPublisher
`-- Runtime
```

RunnerService is the sole Runner owner and applies control commands in order.
Runner owns live input. BtRunner owns replay input and result publication.
Clock remains beside Runtime; its owner passes `now_ms` by value.

## Trading Ownership

```text
Runtime
|-- RuntimeStore
|-- Signaler
|-- Vec<Risk>
`-- BotCycle
    `-- Vec<Executor>
        `-- each Executor owns Vec<Account>
            |-- Venue
            `-- Ledger
                `-- Vec<Trade>
                    `-- Vec<Order>
                        `-- Vec<Fill>
```

Runtime owns work order, not Accounts. BotCycle asks each Executor to reconcile
its Accounts and returns owned snapshots for Runtime Risk evaluation. Runtime
then lets Executors decide. No parent reaches through a child.

## Rust Access Rules

- The owner stores each mutable child directly.
- State changes use the owner's `&mut self`.
- Read-only calls use temporary `&self` borrows.
- Cross-owner state travels as owned values or snapshots.
- Feeds return owned `Bar` or `BboTick` values; they retain no Runtime reference.
- Logger handles share one process logger, not mutable domain state.
- A datastore handle belongs to its store object.
- No global mutable state or long-lived child references.
- No `Arc`, `Rc`, `Mutex`, `RwLock`, `RefCell`, or shared Account without the
  approval required by [Coding Rules](coding/rules.md).

## Independent Modules

`setup` validates and constructs owned values, then transfers them to
Runner/BtRunner. It is not a lifecycle component and owns nothing after return.

`cloid` validates, encodes, and decodes Order identity values. It has no owner,
lifecycle, global state, or mutation.

`common` contains only genuinely shared infrastructure such as errors,
logging, and mechanical executable argument/exit helpers. It does not own
application objects or program-specific parsing and lifecycle.

## Module Layout

```text
src/bin/nuubot-btrunner.rs
src/common.rs
src/common/{error,logging,program}.rs
src/setup.rs
src/cloid.rs
src/runner.rs                         when Runner is ported
src/btrunner.rs
src/runtime.rs
src/clock.rs
src/replay.rs
src/replay/{csv,parquet}.rs
src/signaler.rs
src/signaler/<implementation>.rs
src/risk.rs
src/risk/<implementation>.rs
src/botcycle.rs
src/executor.rs
src/executor/<implementation>.rs
src/account.rs                        when Account is ported
src/venue.rs                          when Venue is ported
src/ledger.rs                         when Ledger is ported
src/{trade,order,fill}.rs             when ported
src/datastore.rs
src/datastore/{models,store,ddl,...}.rs as real boundaries arrive
```

Top-level files are stable core concepts or independent utilities. Each root
module exposes intent; its folder hides implementations and support mechanics.
Do not create empty files or placeholder modules for unported components.
