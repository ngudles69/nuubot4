# BtRunner 30-Month Implementation Audit V1

## Verdict

FAIL.

## Blockers

- `src/replay.rs` lacked the required short intent comments around close-boundary
  validation, normalization, sequence enforcement, and trusted admission.
- `src/bin/nuubot-btrunner.rs` incorrectly said `run()` prints results; printing
  occurs in `main`.

## Accepted Fix

- Added the four caveman intent comments at the admission boundary.
- Corrected the runner comment to describe stop and first-failure preservation.

## Clean Findings

- CSV and Parquet share one admission path; no fallback survives.
- Raw range filtering is unchanged and 2025 timestamp behavior is preserved.
- Focused tests cover mixed precision, sequence, duplicate/gap, invalid fraction,
  and overflow cases.
- The negative harness gate rejects a non-`replay_end` stop.
- The final log contains 200 exact replay summaries, 200 PASS rows, and no
  failure marker.
- No speculative abstraction, dead stub, race risk, or unrelated logic drift
  was found.

Plan-audit v1's early-stop and timestamp mistakes are historical and explicitly
superseded by the current plan.
