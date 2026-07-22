# AGENTS.md

## Purpose

Nuubot4 is the clean Rust port of `D:\rust\nuubot3`.

Nuubot3 is the behavioral source. Its ownership tree, lifecycle phases,
control-loop ordering, configuration meaning, trading decisions, accounting,
risk behavior, persistence evidence, and graceful shutdown are the reference
contracts. Read its current code and owning wiki before porting a component.

Nuubot4 is not a line-by-line translation. Use idiomatic Rust ownership,
enums, structs, traits only where multiple real implementations require them,
typed errors, RAII, and explicit concurrency. Rust-specific structural changes
are allowed when they preserve or improve the documented behavior and make the
affected lifecycle easier to understand and prove.

Do not modify Nuubot3 from a Nuubot4 session unless the user explicitly asks.

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

## Startup

At the start of every session:

1. Read `HANDOFF.md`.
2. Read `wiki/user.md`.
3. Read `wiki/soul.md`.
4. Read every page under `wiki/coding/**` before any coding work.
5. Read `wiki/index.md` and the linked page for the current component.
6. Inspect the corresponding Nuubot3 code and owning Nuubot3 wiki pages.
7. Verify the Nuubot4 branch, status, active processes, and current proof.
8. Restate the intended outcome, scope, files, preserved behavior, and proof;
   wait for user confirmation before changing files or running task commands.

Never guess about Nuubot3 behavior. Cite the exact source code and wiki used.
When Nuubot3 code and docs disagree, stop and report the conflict.

## Default Collaboration Workflow

The root agent stays responsive as controller, delegates execution, and tracks
agents. Use Luna when available, otherwise Sol-low, for fast reading, search,
existence checks, web research, summaries, and bulk simple work. Use Sol-low
for straightforward mechanical edits and explicitly authorized Git operations.
The controller chooses higher reasoning for multi-file logic, diagnosis,
architecture, lifecycle, or risky work.

During interactive review, quickly delegate requested actions or changes while
the root continues the conversation, then verify and integrate the results.
Every agent obeys repository scope, safety, and proof rules. Commit and push
still require explicit user authority.

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

## Rust Design Rules

- Prefer direct structs and functions over frameworks, registries, service
  containers, generic lifecycle walkers, and speculative traits.
- Add a trait only when at least two current implementations require one
  dynamic boundary.
- Prefer enums for closed state and outcome sets.
- Make invalid state difficult to represent, but do not hide lifecycle order
  inside type machinery.
- Use checked conversions at external/config/data boundaries.
- Validate an external payload once, then trust the admitted Rust type.
- Preserve the first error when cleanup also fails.
- Make `stop` idempotent.
- Do not use `unsafe` unless safe Rust cannot meet a measured requirement. Any
  `unsafe` block requires a documented invariant and focused proof.
- Do not introduce PyO3, embedded Python, or a Python fallback.
- Do not use `Arc`, `Rc`, `Mutex`, `RwLock`, or `RefCell` by default. An
  exception requires the user's prior explicit approval for one concrete use,
  with the reason and ownership consequences recorded in the owning wiki before
  implementation. Incidental or unrecorded use is prohibited.
- Do not duplicate Nuubot3 defects merely for textual parity. Report suspected
  defects and obtain approval before changing intended behavior.

## Logging

Logging must exist before instance identity is available. Use one generic
component/module target for early failures. Once Sweep/Bot identity exists,
every Bot-owned object logs with that shared identity so the complete lifecycle
is ordered in one Bot log.

Do not scatter ad hoc output. The stability harness may write one explicit
run-summary log plus failure diagnostics.

## Proof

Use the smallest proof that would fail when the requested behavior is wrong.
Do not run the full test suite without user approval. For execution-affecting
ports, include one real operator-path run in addition to focused tests.

Before claiming parity, compare at least:

- admitted input count and timestamps;
- Clock callback count and order;
- Signal and BotCycle count;
- entry and exit decisions;
- Trade, Order, and Fill evidence;
- Account/Ledger equity and PnL;
- Risk exit and response timing;
- graceful stop order and terminal status;
- persisted and published results.

Do not commit or push without explicit user authority.
