# Coding Style

## Priority

Correctness and proof are mandatory. Among correct solutions, prioritize:

1. Clarity and readability.
2. Simple, visible control flow.
3. Minimal code.
4. Performance when measured.
5. Extensibility only when required.

Write code so the reader understands the intent before reading the
implementation. Prefer clear, symmetrical, boring code over cleverness,
compression, or speculative flexibility.

## Source Order

Order every program and component so readers see progressively deeper detail:

1. Program flow and lifecycle.
2. Context-sensitive domain logic.
3. Context-free mechanical helpers.

Use short comments such as `// Program flow`, `// Domain logic`, and
`// Mechanical helpers` when they materially improve navigation. Do not add
decorative banners, empty sections, or comments where the file is already
obvious.

## Program Flow

- Reserve Program Flow for control flow, ownership, lifecycle, loops, and calls
  to intent-named functions that perform the actual work.
- Do not place verbose mechanical implementation in Program Flow. Move it to
  its owning module or to the domain/helper sections below.
- Make entrypoints, control structures, loops, and key intent functions read
  top-to-bottom like the program's execution.
- Keep executable `main` mechanical. Shared program helpers collect raw
  arguments, handle the final error once, and return the exit code. The
  executable's `program` function should normally read as `parser` then `run`.
- Lower functions propagate errors with `?`. They do not log or handle errors;
  the shared `main` boundary does that once before process exit.
- Keep every lifecycle method that exists contiguous and in canonical order:
  `init -> start -> run/mainloop -> stop`.
- Omit lifecycle phases that have no real work. Never add placeholder methods
  merely to fill the sequence.
- After `start()` succeeds, attempt `stop()` even when `run()` or `mainloop()`
  returns an error. Preserve the primary error; cleanup must not hide it.
- Define private domain calculations and mechanical helpers after the public
  flow even when lifecycle methods call them. A reader must not hunt through
  the file for `start`, `run`, `mainloop`, or `stop`.
- Use explicit branches when they make outcomes clearer than a compressed
  expression.
- Hide mechanics behind the boundary that owns them. Application code says
  `log`; the logging library owns file and console output.
- Preserve stage statistics separately. Do not merge counts from different
  owners; their differences locate failures and inactive stages.
- Put policy in configuration. Do not scatter policy checks through application
  code.

Do not build a Bot log filename inside BtRunner program flow:

```rust
let log_name = format!(
    "nuubot4-bot-{}-{}",
    identity.sweep_id,
    identity.bot_id,
);
let log = logger(Some(&log_name))?;
```

Call the logging helper that owns that mechanical naming rule:

```rust
let log = logger(Some(&bot_log_name(
    identity.sweep_id,
    identity.bot_id,
)))?;
```

## Domain Logic

Domain functions implement application meaning, such as `validate_fills`,
`validate_bbo`, `calc_pnl`, `calc_sharpe`, or `calc_ema`. They follow program
flow because their correctness may change with trading rules, fees, funding,
definitions, or other application context. Domain meaning is not program-level
intent.

## Mechanical Helpers

- Keep private, single-owner helpers at the bottom of their owning file.
- Move a genuinely shared, context-free helper under `common` with a concrete
  owner such as `common/format.rs` or `common/time.rs`.
- Keep errors and logging under the same `common` infrastructure boundary.
- Keep market calendars, candle boundaries, replay timing, price rounding, and
  other domain-sensitive behavior with their domain even when the operation
  looks mechanical.
- Create no placeholder module and no generic `utils` or `helpers` dumping
  ground.

## Naming

- Name functions by responsibility, not current implementation mechanics.
- Keep paired outcomes symmetrical: `success/failure`, `start/stop`, and
  `enable/disable`.
- There is no one-word or two-word rule. Use the shortest unmistakable name.
- Keep outcome logging visible as `log.info(...)` or `log.error(...)`. Do not
  hide one-use outcome branches behind logging wrappers.
- Each component logs its own terminal statistics during `stop()`. Parents pass
  control and errors, not child summaries.
- Log expected work beside completed work. A count without its expected value
  or invariant is not evidence of success.
- Treat a program's argument parser as program flow. Parsing primitives used by
  multiple owners are mechanical helpers.
- Add a named type only when the current values become unclear. For example,
  replace parser tuples with an `Arguments` struct when real additional
  arguments require it, not before.

## Review Check

Hide function bodies and read only names, signatures, and intent comments. The
owner, ordered flow, available lifecycle phases, paired outcomes, and side
effects must still be clear. A missing lifecycle phase must be obvious from the
canonical order. If understanding requires hunting through the file or opening
every helper, improve the structure or names.

Terminal review must also answer:

- What work was expected?
- What work completed?
- What proves success or failure?
- Which owner logs that evidence?
