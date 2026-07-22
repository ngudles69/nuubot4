# Coding Rules

These are the general rules. Component wiki pages own specific behavior.
Nuubot4 design wins where Nuubot3 differs.

## Before Editing

- State the outcome, scope, affected owners/files, preserved behavior, and proof.
- Read the owning design and trace the real creator-to-effect flow first.
- Fix the owning boundary once. Do not patch symptoms or expand scope silently.
- Report suspected Nuubot3 defects; do not port them without approval.

## Shape And Ownership

- Call across boundaries at task-level intent. The callee owns mechanics.
- Each mutable object has one direct owner. Parents manage direct children only.
- Never reach through a child. Add one narrow intent method or owned snapshot.
- Keep lifecycle order visible and top-to-bottom:
  `init -> start -> run/mainloop -> stop`.
- Put work in its real phase. Omit empty or fake lifecycle methods.
- Stop admission first, then unwind direct children in safe reverse order.
- Make `stop` idempotent and preserve the first failure during cleanup.
- Prefer direct structs/functions and enums. Add a trait only for two current
  implementations that need the same boundary.
- Do not add containers, registries, factories, generic lifecycle walkers, or
  shared ownership for hypothetical needs.
- Use concrete domain names. Avoid vague `helper`, `utils`, `process`, or
  `manager` names that hide mixed responsibilities.

## File Layout

- Keep real executable entrypoints under `src/bin/`.
- Keep core concepts as top-level `src/<object>.rs` files.
- Group related support files under a named `src/<group>/` folder when that
  hides useful detail. Use `<group>.rs` as the module boundary.

```text
src/bin/nuubot-btrunner.rs
src/runtime.rs
src/common.rs
src/common/logging.rs
src/common/error.rs
src/setup.rs
src/cloid.rs
src/datastore.rs
src/datastore/models.rs
src/datastore/sweep.rs
src/signaler.rs
src/signaler/macross.rs
src/risk.rs
src/risk/balanced.rs
src/executor.rs
src/executor/observer.rs
```

Keep `src/bin/` for executables. Do not scatter one group's support files.

## Caveman Intent Comments

- Put short, blunt intent comments before meaningful code blocks.
- Use caveman prose: usually two to five words.
- Describe purpose, state change, or ordering reason. Never narrate Rust syntax.
- A reader hiding implementation statements must still see the L1 flow from
  the signature, `///` docs, and block comments.
- Do not comment trivial statements or repeat the function name.

```rust
// Reject stale input
// Reconcile account truth
// Evaluate risk
// Execute decisions
// Unwind children
```

## Trust And Failure

- Validate config, files, rows, and external payloads once at admission.
- Reject complete invalid input before mutation, persistence, or external calls.
- Convert admitted input into trusted Rust types; do not revalidate downstream.
- Treat broken internal invariants as producer bugs. Fail and fix the producer.
- Propagate unexpected errors. Do not silently retry, skip, repair, default,
  fall back, or report partial success without an approved recovery contract.
- Preserve timestamp, identity, and external-outcome meaning; never invent facts.

## Keep It Small

- Prefer deletion, existing code, the standard library, and direct code.
- Add no speculative abstractions, fields, config, caches, adapters, or bridges.
- Keep persistence outside domain objects and formatting outside domain logic.
- Add complexity only for required behavior, safety, ownership, or measured need.

## Logging

- Log early failures through one generic module target.
- Once Bot identity exists, every owned component uses that shared Bot log.
- Do not scatter ad hoc output or create per-component Bot logs.

## Proof

- Use the smallest proof that fails when the changed behavior is wrong.
- Execution changes need focused proof plus one real operator-path run.
- Compare relevant counts, timestamps, ordered effects, stop reason, exit status,
  and durable evidence.
- Never run the full suite without approval or claim a gate not completed.
- Before completion, inspect the whole diff against applicable coding and design
  pages; passing tests do not excuse a known contract violation.
