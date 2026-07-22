# Rust Port Contract

The [Project](project.md) owns purpose, language, workspace boundaries, port
order, lifecycle, and parity requirements. This page owns the behavioral-source
contract, port architecture, and durable stability evidence.

## Source Of Truth

For each component, read its current Nuubot3 code plus the wiki page that owns
its invariant. Do not port from memory or from obsolete numbered/backup files.

When code and documentation disagree:

1. record the exact conflict;
2. identify which path is currently reachable and proven;
3. ask the user which behavior Nuubot4 should own;
4. document the decision before implementation.

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

## Proven Stability Slice

The current code contains only the temporary control Runtime, MacrossSignaler,
BalancedRisk, ControlBotCycle, and ObserverExecutor. No Account, Ledger, trading
Executor, Venue, or persistence behavior is wired. The initial three-month gate
proved both CSV and Parquet. The extended gate proves 30 months of Parquet in
fresh standalone Windows processes.

Acceptance:

```text
ticks = 7,948,800
timer callbacks = 794,880
CSV 200/200 pass
Parquet 200/200 pass

30-month Parquet ticks = 78,883,200
30-month timer callbacks = 7,888,320
first UTC timestamp = 1701388801000
last UTC timestamp = 1780272000000
30-month Parquet = 200/200 pass
one-second pause between fresh processes
```

CSV and Arrow Parquet are external replay boundaries. Both validate strict
shape, types, values, and one-second sequence once, then emit trusted
`BboTick` values. A raw one-second close must fall in the final 1,000
microseconds before its next UTC second and is normalized to that boundary.
Arrow types never enter Runtime or the trading domain.

## Later Parity

Port one complete owner-to-effect path at a time. Every stage must compare its
observable results with Nuubot3 before the next descendant is added. The final
Account-stack port must reproduce the accepted fixed Sweep evidence, not merely
complete without crashing.
