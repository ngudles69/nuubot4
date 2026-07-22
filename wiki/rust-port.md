# Rust Port Contract

## Objective

Port Nuubot3 into a fully Rust application with the same observable trading,
accounting, Risk, persistence, recovery, replay, and shutdown behavior.

The port may use different internal structures where Rust provides a clearer
solution. Behavior and ownership are the parity boundary, not Python class or
line layout.

## Source Of Truth

For each component, read its current Nuubot3 code plus the wiki page that owns
its invariant. Do not port from memory or from obsolete numbered/backup files.

When code and documentation disagree:

1. record the exact conflict;
2. identify which path is currently reachable and proven;
3. ask the user which behavior Nuubot4 should own;
4. document the decision before implementation.

## Workspace Contract

Nuubot4 stores all writable state inside
`D:\rust\nuubot4\workspace`. This includes configuration, SQLite databases,
logs, results, caches, and temporary files.

The sole shared external dependency is read-only market data under
`D:\workspace\data`. Read the same data as Nuubot3; never rewrite it merely to
make the Rust port pass.

Use `nuubot4` names for datastore and artifact identities that could otherwise
be confused:

```text
workspace/datastore/nuubot4_sweeps.db
workspace/datastore/nuubot4_bot_<sweep_id>_<bot_id>.db
PostgreSQL schema nuubot4, when later required
```

Never open a Nuubot3 datastore for writes.

## Architecture

```text
Runner or BtRunner
-> Clock and input driver
-> Runtime
-> Signaler and Risk
-> active BotCycle
-> Executor
-> Account
-> Venue: Simulator or Hyperliquid
-> Ledger -> Trade -> Order -> Fill transaction evidence
```

Parents own direct children and call only their direct lifecycle. Runtime owns
the coherent Account reconciliation barrier. Each Executor owns its own
`Vec<Account>` and reconciles those Accounts through the shared Executor trait
method. Runtime receives owned Account snapshots for Risk, then allows Executor
decisions. See [Recon Ownership and Flow](recon.md).

## Lifecycle

```text
init
    create and initialize owned children
    leave drivers stopped

start
    establish initial truth
    perform real ready-to-running transitions
    open admission last

run or mainloop
    drive one long-lived input loop or one bounded decision pass

stop
    close admission first
    quiesce input
    unwind children safely
    publish terminal evidence once
```

Rust destructors are a safety net for resources, not a replacement for visible
domain shutdown.

## Proven Stability Slice

The current code contains only the temporary control Runtime, MacrossSignaler,
BalancedRisk, ControlBotCycle, and ObserverExecutor. No Account, Ledger, trading
Executor, Venue, or persistence behavior is wired. It proves that a standalone
Rust process can repeatedly consume the exact three-month dataset through both
CSV and Parquet and produce the expected tick and timer counts on Windows.

Acceptance:

```text
ticks = 7,948,800
timer callbacks = 794,880
CSV 200/200 pass
Parquet 200/200 pass
one-second pause between fresh processes
```

CSV and Arrow Parquet are external replay boundaries. Both validate strict
shape, types, values, and one-second sequence once, then emit trusted
`BboTick` values. Arrow types never enter Runtime or the trading domain.

## Later Parity

Port one complete owner-to-effect path at a time. Every stage must compare its
observable results with Nuubot3 before the next descendant is added. The final
Account-stack port must reproduce the accepted fixed Sweep evidence, not merely
complete without crashing.
