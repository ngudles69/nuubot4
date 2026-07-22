# Coding Rules

These rules bias toward caution over speed. Use judgment for trivial tasks.
Component pages own component behavior; this page owns universal implementation
discipline.

## 1. Think Before Coding

Do not assume or hide confusion. Surface tradeoffs.

Before implementing:

- State assumptions explicitly. If uncertain, ask.
- Present materially different interpretations; do not choose silently.
- Say when a simpler approach exists. Push back when warranted.
- If something is unclear, stop, name the confusion, and ask.
- Identify the canonical owner for every intended change before editing.
- Read the surrounding code and docs so the change is coherent, not piecemeal.
- Make a piecemeal edit only when the user explicitly requests one.
- Restate the outcome, scope, affected owners/files, preserved behavior, and
  proof, then wait for confirmation.
- Read the owning design and trace the real creator-to-effect flow.
- Fix the owning boundary once. Do not patch symptoms or expand scope silently.
- Report suspected Nuubot3 defects; do not port them without approval.

## 2. Simplicity First

Write the minimum code that solves the problem. Nothing speculative.

- Add no feature beyond what was asked.
- Add no abstraction for single-use code.
- Add no flexibility or configurability that was not requested.
- Add no error handling for impossible scenarios.
- Prefer deletion, existing code, the standard library, and direct code.
- Add no speculative fields, config, caches, adapters, or bridges.
- Keep persistence outside domain objects and formatting outside domain logic.
- Add complexity only for required behavior, safety, ownership, or measured
  need.
- If 200 lines could be 50, rewrite it.
- Ask whether a senior engineer would call it overcomplicated. If yes, simplify.

## 3. Surgical Changes

Touch only what is required. Clean up only your own mess.

- Do not improve adjacent code, comments, formatting, or names.
- Do not refactor code that is not broken.
- Match the existing style even when another style is preferable.
- Mention unrelated dead code; do not delete it.
- Remove imports, variables, functions, and files made unused by your change.
- Do not remove pre-existing dead code unless asked.
- Every changed line must trace directly to the user's request.

## 4. Goal-Driven Execution

Define success criteria and loop until they are verified.

- Turn vague tasks into observable outcomes before implementation.
- For validation, prove invalid inputs fail as intended.
- For a bug, write the smallest check that reproduces it, then make it pass.
- For a refactor, verify behavior before and after.
- For multi-step work, state a brief plan with a verification check per step.
- Use the smallest proof that fails when the changed behavior is wrong.
- Execution changes need focused proof plus one real operator-path run.
- Never run the full suite without user approval.
- Inspect the complete diff against the owning docs before completion.
- Never claim a gate that was not completed. Passing tests do not excuse a
  known contract violation.
- Loop until every stated success criterion is verified.

Use the [Project Parity Proof](../project.md#parity-proof) for parity work.

## Rust Ownership and Lifecycle

- Call across boundaries at task-level intent. The callee owns mechanics.
- Each mutable object has one direct owner. Parents manage direct children only.
- Never reach through a child. Add one narrow intent method or owned snapshot.
- Keep lifecycle order visible: `init -> start -> run/mainloop -> stop`.
- Put work in its real phase. Omit empty or fake lifecycle methods.
- Stop admission first, then unwind direct children in safe reverse order.
- Make `stop` idempotent and preserve the first failure during cleanup.
- Prefer direct structs and functions. Use enums for closed state and outcomes.
- Add a trait only when two current implementations need the same boundary.
- Make invalid state difficult to represent without hiding lifecycle order.
- Do not add containers, registries, factories, generic lifecycle walkers, or
  shared ownership for hypothetical needs.
- Do not use `Arc`, `Rc`, `Mutex`, `RwLock`, or `RefCell` by default. One
  concrete exception requires the user's prior approval and an ownership note
  in the owning wiki before implementation.
  Incidental or unrecorded use is prohibited.
- Use concrete domain names. Avoid vague `helper`, `utils`, `process`, or
  `manager` names that hide mixed responsibility.

## File Layout

- Keep executable entrypoints under `src/bin/`.
- Keep stable core concepts as top-level `src/<object>.rs` files.
- Put one concept's implementations and support under `src/<concept>/`.
- Use the top-level `<concept>.rs` as that module's intent boundary.
- Do not create placeholder files or modules for unported components.

See [Ownership and Project Structure](../ownership.md) for the current map.

## Intent Comments

- Put short, blunt intent comments before meaningful code blocks.
- Use caveman prose, usually two to five words.
- Describe purpose, state change, or ordering reason; never narrate Rust syntax.
- A reader hiding statements must still see the L1 flow from signatures, docs,
  and block comments.
- Do not comment trivial statements or repeat the function name.

```rust
// Reject stale input
// Reconcile account truth
// Evaluate risk
// Execute decisions
// Unwind children
```

## Trust and Failure

- Use checked conversions at config, data, datastore, and external boundaries.
- Use typed errors for recoverable failures.
- Validate external input once, then trust the admitted Rust type.
- Reject complete invalid input before mutation, persistence, or external calls.
- Treat broken internal invariants as producer bugs. Fail and fix the producer.
- Propagate unexpected errors. Do not silently retry, skip, repair, default,
  fall back, or report partial success without an approved recovery contract.
- Preserve timestamp, identity, and external-outcome meaning; never invent facts.
- Do not use `unsafe` unless safe Rust cannot meet a measured requirement. Each
  `unsafe` block requires a documented invariant and focused proof.
- Do not introduce PyO3, embedded Python, or a Python fallback.

## Logging

- Log early failures through one generic module target.
- Once Bot identity exists, every owned component uses that shared Bot log.
- Do not scatter ad hoc output or create per-component Bot logs.
- Always write logs to file. `logging.console` controls only the console mirror.
- The stability harness may write one run-summary log plus failure diagnostics.

## Authority

- Do not modify Nuubot3 unless the user explicitly asks.
- Do not commit or push without explicit user authority.
