# BtRunner 30-Month Plan Audit v1

Result: **FAIL**

- The planned 30-month counts and timestamps are correct: 78,883,200 ticks,
  7,888,320 callbacks, `1701360001000` first, `1780243200000` last.
- Observer `max_ticks` is currently 10,000,000, so the replay stops early.
- BtRunner skips exact tick, callback, and timestamp validation when Runtime
  stops before replay end.
- `rtest.sh` currently treats process exit alone as success.
- Required fix: each run must fail unless the complete replay reaches its exact
  ticks, callbacks, timestamps, and `stop_reason=replay_end`.

