# BtRunner Runtime Plan Audit V1

Result: FAIL, corrected before implementation.

## Findings

1. Replay was modeled as an owned lifecycle object instead of a utility
   iterator boundary.
2. `nuubot_setup` did not define the returned ownership shape.
3. The required early and Bot-identity logging path was absent.
4. The repeated-process proof did not specify a run count or exact evidence.

## Disposition

All findings were accepted. The plan now makes replay a public iterator
function, defines an owned `SetupContext`, specifies generic-then-identity
logging, and requires 20 fresh release processes per loader with complete
per-run evidence before any larger gate.
