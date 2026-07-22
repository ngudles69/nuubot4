# Nuubot4 Project

## Purpose

Nuubot4 is the clean Rust port of `D:\rust\nuubot3`.

Nuubot3 is the behavioral source. Its ownership tree, lifecycle phases,
control-loop ordering, configuration meaning, trading decisions, accounting,
risk behavior, persistence evidence, and graceful shutdown are the reference
contracts. Read its current code and owning wiki before porting a component.

Nuubot4 is not a line-by-line translation. Rust-specific structures are
allowed when they preserve or improve the documented behavior and make the
affected lifecycle easier to understand and prove.
Behavior and ownership are the parity boundary, not Python class or line
layout.

Do not modify Nuubot3 from a Nuubot4 session unless the user explicitly asks.

## Language

Nuubot4 is a fully Rust application using Rust edition 2024. Use idiomatic Rust
ownership, enums, structs, typed errors, RAII, and explicit concurrency. The
canonical implementation standards are in [Coding Rules](coding/rules.md).

## Workspace Boundary

Nuubot4 owns all writable application state inside this repository:

```text
D:\rust\nuubot4\workspace\config
D:\rust\nuubot4\workspace\datastore
D:\rust\nuubot4\workspace\logs
D:\rust\nuubot4\workspace\results
D:\rust\nuubot4\workspace\temp
```

The only external workspace dependency is shared read-only market data under
`D:\workspace\data`. Nuubot4 reads the same source data as Nuubot3 so parity
uses identical inputs.

Nuubot4 accepts the same compatible configuration meaning, but its config files
live under the Nuubot4 repository workspace. SQLite files, logs, results,
caches, and temporary files never use Nuubot3 workspace paths. Every writable
artifact begins with `nuubot4` where a filename or datastore identity can be
shared or confused. If PostgreSQL is later required, use an isolated
`nuubot4` schema.

```text
workspace/datastore/nuubot4_sweeps.db
workspace/datastore/nuubot4_bot_<sweep_id>_<bot_id>.db
PostgreSQL schema nuubot4, when later required
```

Never open a Nuubot3 datastore for writes.

## Initial Objective

Build the smallest fully Rust stability harness before porting the Bot:

```text
read the same BTC CSV data
-> calendar-week windows
-> 7,948,800 one-second ticks
-> one TickClock callback per 10 seconds
-> exactly 794,880 callbacks
-> one standalone OS process per run
-> 20 clean runs
-> 100 clean runs
-> 200 clean runs
```

Preserve per-run elapsed time, pass/fail, exit status, and failure diagnostics.
Use a one-second pause between processes. Stop on any failed gate and diagnose
before increasing the run count.

The first harness proves Rust/Windows data and process stability only. It does
not claim trading parity.

## Port Order

Port in complete vertical stages:

1. CSV replay reader and TickClock stability harness.
2. BtRunner lifecycle and replay supervision.
3. Runtime, temporary control BotCycle, MacrossSignaler, ObserverExecutor, and
   BalancedRisk.
4. Canonical BotCycle and real Signaler/Executor/Risk behavior.
5. Account, Ledger, Simulator, Trade, Order, and Fill.
6. Datastores, result publication, recovery, and live Runner.

At every stage reproduce the applicable Nuubot3 inputs, ordered effects,
counts, timestamps, stop reasons, results, and failure behavior before moving
down the ownership tree.

## Canonical Lifecycle

Use the Nuubot3 lifecycle as the default organization:

```text
init       create and initialize the owned subtree; nothing runs
start      perform real ready-to-running transitions; no control loop
run        own and supervise a long-lived driver
mainloop   perform one bounded repeated decision pass
stop       close admission, quiesce input, and unwind direct children safely
```

`init` leaves drivers stopped. `start` establishes initial truth and opens
admission last. `stop` publishes terminal evidence once.

Each component:

- creates its direct children;
- initializes its direct children;
- starts only children with real start work;
- calls its direct children during bounded execution;
- stops its direct children in safe reverse order.

Executor is event-driven and has no independent task or internal loop.
Runtime reconciles Account truth before Risk and Executor decisions. A
completed Executor and a Risk exit use one Runtime-owned graceful Bot stop
route.

Rust RAII may perform guaranteed resource release, but domain stop ordering,
final reconciliation, durable evidence, and explicit terminal state remain
visible lifecycle work.

## Parity Proof

Before claiming parity, compare every applicable item:

- admitted input count and timestamps;
- Clock callback count and order;
- Signal and BotCycle count;
- entry and exit decisions;
- Trade, Order, and Fill evidence;
- Account/Ledger equity and PnL;
- Risk exit and response timing;
- graceful stop order and terminal status;
- persisted and published results.

The detailed Rust port, ownership, and current-state evidence live in
[Rust Port Contract](rust-port.md), [Ownership](ownership.md), and
[HANDOFF](../HANDOFF.md).
